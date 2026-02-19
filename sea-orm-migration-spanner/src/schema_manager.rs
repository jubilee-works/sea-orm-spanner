use gcloud_googleapis::spanner::admin::database::v1::UpdateDatabaseDdlRequest;
use gcloud_spanner::admin::client::Client as AdminClient;
use gcloud_spanner::admin::AdminClientConfig;
use regex::Regex;
use sea_orm::sea_query::{
    backend::MysqlQueryBuilder, IndexCreateStatement, IndexDropStatement, TableAlterStatement,
    TableCreateStatement, TableDropStatement,
};
use sea_orm::DbErr;
use sea_query_spanner::{SpannerAlterTable, SpannerIndexBuilder, SpannerTableBuilder};

/// Convert MySQL DDL to Spanner DDL
///
/// Handles type mappings and syntax differences:
/// - `INT` / `INTEGER` → `INT64`
/// - `BIGINT` → `INT64`
/// - `SMALLINT` / `TINYINT` → `INT64`
/// - `VARCHAR(N)` / `CHAR(N)` → `STRING(N)`
/// - `TEXT` / `LONGTEXT` / `MEDIUMTEXT` → `STRING(MAX)`
/// - `BOOLEAN` / `BOOL` / `TINYINT(1)` → `BOOL`
/// - `FLOAT` / `DOUBLE` / `REAL` → `FLOAT64`
/// - `DECIMAL(p,s)` / `NUMERIC(p,s)` → `NUMERIC` (Spanner supports NUMERIC)
/// - `DATETIME` / `TIMESTAMP` → `TIMESTAMP`
/// - `DATE` → `DATE`
/// - `BINARY(16)` → `UUID` (SeaORM MySQL backend uses BINARY(16) for Uuid)
/// - `BLOB` / `BINARY` / `VARBINARY` → `BYTES(MAX)`
/// - `JSON` → `JSON`
/// - Removes `AUTO_INCREMENT`
/// - Removes `ENGINE=...`, `CHARSET=...`, `COLLATE=...`
/// - Removes `DEFAULT ...` (Spanner doesn't support DEFAULT in CREATE TABLE)
/// - Converts inline `PRIMARY KEY` to Spanner's trailing `PRIMARY KEY (col)` syntax
fn mysql_ddl_to_spanner(mysql_ddl: &str) -> String {
    let mut sql = mysql_ddl.to_string();

    let if_not_exists_re = Regex::new(r"(?i)\s*IF\s+NOT\s+EXISTS").unwrap();
    sql = if_not_exists_re.replace_all(&sql, "").to_string();

    let if_exists_re = Regex::new(r"(?i)\s*IF\s+EXISTS").unwrap();
    sql = if_exists_re.replace_all(&sql, "").to_string();

    let auto_inc_re = Regex::new(r"(?i)\s*AUTO_INCREMENT").unwrap();
    sql = auto_inc_re.replace_all(&sql, "").to_string();

    // Remove ENGINE clause
    let engine_re = Regex::new(r"(?i)\s*ENGINE\s*=\s*\w+").unwrap();
    sql = engine_re.replace_all(&sql, "").to_string();

    // Remove CHARSET clause
    let charset_re = Regex::new(r"(?i)\s*(DEFAULT\s+)?CHARSET\s*=\s*\w+").unwrap();
    sql = charset_re.replace_all(&sql, "").to_string();

    // Remove CHARACTER SET clause
    let char_set_re = Regex::new(r"(?i)\s*CHARACTER\s+SET\s+\w+").unwrap();
    sql = char_set_re.replace_all(&sql, "").to_string();

    // Remove COLLATE clause
    let collate_re = Regex::new(r"(?i)\s*COLLATE\s*=?\s*\w+").unwrap();
    sql = collate_re.replace_all(&sql, "").to_string();

    // Remove DEFAULT values (Spanner doesn't support DEFAULT in CREATE TABLE for most types)
    // Be careful to not remove DEFAULT CURRENT_TIMESTAMP patterns
    let default_re = Regex::new(r"(?i)\s*DEFAULT\s+(?:'[^']*'|\d+|NULL|TRUE|FALSE)").unwrap();
    sql = default_re.replace_all(&sql, "").to_string();

    // Remove UNSIGNED (Spanner INT64 is always signed)
    let unsigned_re = Regex::new(r"(?i)\s+UNSIGNED").unwrap();
    sql = unsigned_re.replace_all(&sql, "").to_string();

    let unique_key_re = Regex::new(r"(?i)\s+UNIQUE(\s+KEY)?").unwrap();
    sql = unique_key_re.replace_all(&sql, "").to_string();

    // Type conversions (order matters - more specific patterns first)

    // TINYINT(1) → BOOL (MySQL boolean pattern)
    let tinyint1_re = Regex::new(r"(?i)\bTINYINT\s*\(\s*1\s*\)").unwrap();
    sql = tinyint1_re.replace_all(&sql, "BOOL").to_string();

    let int_types_re = Regex::new(r"(?i)\b(BIG)?INT(EGER)?\b\s*(\(\s*\d+\s*\))?").unwrap();
    sql = int_types_re.replace_all(&sql, "INT64 ").to_string();

    let smallint_re = Regex::new(r"(?i)\b(SMALL|TINY|MEDIUM)INT\b\s*(\(\s*\d+\s*\))?").unwrap();
    sql = smallint_re.replace_all(&sql, "INT64 ").to_string();

    // VARCHAR/CHAR → STRING
    let varchar_re = Regex::new(r"(?i)\b(VAR)?CHAR\s*\(\s*(\d+)\s*\)").unwrap();
    sql = varchar_re.replace_all(&sql, "STRING($2)").to_string();

    // TEXT types → STRING(MAX)
    let text_re = Regex::new(r"(?i)\b(LONG|MEDIUM)?TEXT").unwrap();
    sql = text_re.replace_all(&sql, "STRING(MAX)").to_string();

    // BOOLEAN/BOOL → BOOL
    let bool_re = Regex::new(r"(?i)\bBOOL(EAN)?").unwrap();
    sql = bool_re.replace_all(&sql, "BOOL").to_string();

    // FLOAT/DOUBLE/REAL → FLOAT64
    let float_re =
        Regex::new(r"(?i)\b(FLOAT|DOUBLE|REAL)(\s*\(\s*\d+\s*(,\s*\d+\s*)?\))?").unwrap();
    sql = float_re.replace_all(&sql, "FLOAT64").to_string();

    // DECIMAL/NUMERIC → NUMERIC (Spanner supports this)
    let decimal_re =
        Regex::new(r"(?i)\b(DECIMAL|NUMERIC)\s*(\(\s*\d+\s*(,\s*\d+\s*)?\))?").unwrap();
    sql = decimal_re.replace_all(&sql, "NUMERIC").to_string();

    // DATETIME → TIMESTAMP
    let datetime_re = Regex::new(r"(?i)\bDATETIME\s*(\(\s*\d+\s*\))?").unwrap();
    sql = datetime_re.replace_all(&sql, "TIMESTAMP ").to_string();

    let timestamp_re = Regex::new(r"(?i)\bTIMESTAMP\s*(\(\s*\d+\s*\))?").unwrap();
    sql = timestamp_re.replace_all(&sql, "TIMESTAMP ").to_string();

    // DATE stays DATE

    // BLOB/BINARY/VARBINARY → BYTES
    let blob_re = Regex::new(r"(?i)\b(LONG|MEDIUM|TINY)?BLOB").unwrap();
    sql = blob_re.replace_all(&sql, "BYTES(MAX)").to_string();

    // BINARY(16) → UUID (SeaORM MySQL backend generates BINARY(16) for Uuid type)
    let binary16_re = Regex::new(r"(?i)\bBINARY\s*\(\s*16\s*\)").unwrap();
    sql = binary16_re.replace_all(&sql, "UUID").to_string();

    let binary_re = Regex::new(r"(?i)\b(VAR)?BINARY\s*\(\s*(\d+)\s*\)").unwrap();
    sql = binary_re.replace_all(&sql, "BYTES($2)").to_string();

    // JSON stays JSON (Spanner supports JSON)

    let inline_pk_re =
        Regex::new(r"(?i)`?(\w+)`?\s+(\w+(?:\s*\([^)]*\))?)\s+NOT\s+NULL\s+PRIMARY\s+KEY").unwrap();
    let pk_col = if let Some(caps) = inline_pk_re.captures(&sql) {
        let col_name = caps.get(1).unwrap().as_str().to_string();
        let col_type = caps.get(2).unwrap().as_str().to_string();
        sql = inline_pk_re
            .replace(&sql, &format!("`{}` {} NOT NULL", col_name, col_type))
            .to_string();
        Some(col_name)
    } else {
        None
    };

    let multi_space_re = Regex::new(r"  +").unwrap();
    sql = multi_space_re.replace_all(&sql, " ").to_string();

    let trailing_comma_re = Regex::new(r",\s*\)").unwrap();
    sql = trailing_comma_re.replace_all(&sql, ")").to_string();

    if let Some(col_name) = pk_col {
        if !sql.to_uppercase().contains("PRIMARY KEY (")
            && !sql.to_uppercase().contains("PRIMARY KEY(")
        {
            if let Some(pos) = sql.rfind(')') {
                sql = format!(
                    "{}) PRIMARY KEY (`{}`)",
                    &sql[..pos].trim_end_matches(',').trim_end(),
                    col_name
                );
            }
        }
    }

    sql.trim().to_string()
}

/// Schema manager for Spanner migrations
///
/// Provides methods to execute DDL statements against Spanner.
/// Supports both raw DDL strings and builder patterns.
pub struct SchemaManager {
    database_path: String,
}

impl SchemaManager {
    /// Create a new SchemaManager
    pub fn new(database_path: &str) -> Self {
        Self {
            database_path: database_path.to_string(),
        }
    }

    /// Get the database path
    pub fn database_path(&self) -> &str {
        &self.database_path
    }

    /// Execute multiple DDL statements
    pub async fn execute_ddl(&self, statements: Vec<String>) -> Result<(), DbErr> {
        let admin_config = if std::env::var("SPANNER_EMULATOR_HOST").is_ok() {
            AdminClientConfig::default()
        } else {
            AdminClientConfig::default()
                .with_auth()
                .await
                .map_err(|e| DbErr::Custom(format!("Failed to authenticate with GCP: {}", e)))?
        };
        let admin_client = AdminClient::new(admin_config)
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to create admin client: {}", e)))?;

        let result = admin_client
            .database()
            .update_database_ddl(
                UpdateDatabaseDdlRequest {
                    database: self.database_path.clone(),
                    statements,
                    operation_id: "".to_string(),
                    proto_descriptors: vec![],
                    throughput_mode: false,
                },
                None,
            )
            .await;

        match result {
            Ok(mut op) => {
                op.wait(None)
                    .await
                    .map_err(|e| DbErr::Custom(format!("DDL operation failed: {}", e)))?;
                Ok(())
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("AlreadyExists")
                    || err_str.contains("already exists")
                    || err_str.contains("Duplicate name")
                {
                    Ok(())
                } else {
                    Err(DbErr::Custom(format!("DDL execution failed: {}", err_str)))
                }
            }
        }
    }

    /// Execute a single DDL statement
    pub async fn execute(&self, ddl: &str) -> Result<(), DbErr> {
        self.execute_ddl(vec![ddl.to_string()]).await
    }

    // ========================================
    // Table Operations
    // ========================================

    /// Create a table using raw DDL
    pub async fn create_table_raw(&self, ddl: &str) -> Result<(), DbErr> {
        self.execute_ddl(vec![ddl.to_string()]).await
    }

    pub async fn create_table(&self, stmt: TableCreateStatement) -> Result<(), DbErr> {
        let sql = stmt.build(MysqlQueryBuilder);
        let spanner_sql = mysql_ddl_to_spanner(&sql);
        self.execute_ddl(vec![spanner_sql]).await
    }

    pub async fn create_table_spanner(&self, builder: SpannerTableBuilder) -> Result<(), DbErr> {
        self.execute_ddl(vec![builder.build()]).await
    }

    pub async fn drop_table(&self, stmt: TableDropStatement) -> Result<(), DbErr> {
        let sql = stmt.build(MysqlQueryBuilder);
        let spanner_sql = mysql_ddl_to_spanner(&sql);
        self.execute_ddl(vec![spanner_sql]).await
    }

    pub async fn drop_table_by_name(&self, table_name: &str) -> Result<(), DbErr> {
        self.execute_ddl(vec![format!("DROP TABLE {}", table_name)])
            .await
    }

    pub async fn drop_table_if_exists(&self, table_name: &str) -> Result<(), DbErr> {
        let result = self.drop_table_by_name(table_name).await;
        match result {
            Ok(()) => Ok(()),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("not found")
                    || err_str.contains("does not exist")
                    || err_str.contains("NOT_FOUND")
                {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    // ========================================
    // Index Operations
    // ========================================

    pub async fn create_index_raw(&self, ddl: &str) -> Result<(), DbErr> {
        self.execute_ddl(vec![ddl.to_string()]).await
    }

    pub async fn create_index_spanner(&self, builder: SpannerIndexBuilder) -> Result<(), DbErr> {
        self.execute_ddl(vec![builder.build()]).await
    }

    pub async fn create_index(&self, stmt: IndexCreateStatement) -> Result<(), DbErr> {
        let sql = stmt.build(MysqlQueryBuilder);
        let spanner_sql = mysql_ddl_to_spanner(&sql);
        self.execute_ddl(vec![spanner_sql]).await
    }

    pub async fn drop_index(&self, stmt: IndexDropStatement) -> Result<(), DbErr> {
        let sql = stmt.build(MysqlQueryBuilder);
        let spanner_sql = mysql_ddl_to_spanner(&sql);
        self.execute_ddl(vec![spanner_sql]).await
    }

    pub async fn drop_index_by_name(&self, index_name: &str) -> Result<(), DbErr> {
        self.execute_ddl(vec![format!("DROP INDEX {}", index_name)])
            .await
    }

    pub async fn drop_index_if_exists(&self, index_name: &str) -> Result<(), DbErr> {
        let result = self.drop_index_by_name(index_name).await;
        match result {
            Ok(()) => Ok(()),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("not found")
                    || err_str.contains("does not exist")
                    || err_str.contains("NOT_FOUND")
                {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    pub async fn alter_table(&self, stmt: TableAlterStatement) -> Result<(), DbErr> {
        let sql = stmt.build(MysqlQueryBuilder);
        let spanner_sql = mysql_ddl_to_spanner(&sql);
        self.execute_ddl(vec![spanner_sql]).await
    }

    pub async fn alter_table_spanner(&self, alter: SpannerAlterTable) -> Result<(), DbErr> {
        self.execute_ddl(vec![alter.build()]).await
    }

    pub async fn add_column(
        &self,
        table: &str,
        column_name: &str,
        column_type: &str,
        not_null: bool,
    ) -> Result<(), DbErr> {
        let alter = SpannerAlterTable::add_column(table, column_name, column_type, not_null);
        self.execute_ddl(vec![alter.build()]).await
    }

    pub async fn drop_column(&self, table: &str, column_name: &str) -> Result<(), DbErr> {
        let alter = SpannerAlterTable::drop_column(table, column_name);
        self.execute_ddl(vec![alter.build()]).await
    }
}
