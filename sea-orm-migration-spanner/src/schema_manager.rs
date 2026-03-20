use {
    gcloud_gax::conn::{ConnectionManager, ConnectionOptions},
    gcloud_googleapis::spanner::admin::database::v1::UpdateDatabaseDdlRequest,
    gcloud_longrunning::autogen::operations_client::OperationsClient,
    gcloud_spanner::{
        admin::{database::database_admin_client::DatabaseAdminClient, AdminClientConfig},
        apiv1::conn_pool::{AUDIENCE, SPANNER},
    },
    regex::Regex,
    sea_orm::{
        sea_query::{
            backend::MysqlQueryBuilder, IndexCreateStatement, IndexDropStatement,
            TableAlterStatement, TableCreateStatement, TableDropStatement,
        },
        ConnectionTrait, DatabaseConnection, DbErr, ExecResult,
    },
    sea_orm_spanner::SpannerDatabase,
    sea_query_spanner::{SpannerAlterTable, SpannerIndexBuilder, SpannerTableBuilder},
    std::{sync::LazyLock, time::Duration},
};

macro_rules! regex {
    ($pattern:expr) => {
        LazyLock::new(|| Regex::new($pattern).expect(concat!("invalid regex: ", $pattern)))
    };
}

static RE_IF_NOT_EXISTS: LazyLock<Regex> = regex!(r"(?i)\s*IF\s+NOT\s+EXISTS");
static RE_IF_EXISTS: LazyLock<Regex> = regex!(r"(?i)\s*IF\s+EXISTS");
static RE_AUTO_INCREMENT: LazyLock<Regex> = regex!(r"(?i)\s*AUTO_INCREMENT");
static RE_ENGINE: LazyLock<Regex> = regex!(r"(?i)\s*ENGINE\s*=\s*\w+");
static RE_CHARSET: LazyLock<Regex> = regex!(r"(?i)\s*(DEFAULT\s+)?CHARSET\s*=\s*\w+");
static RE_CHARACTER_SET: LazyLock<Regex> = regex!(r"(?i)\s*CHARACTER\s+SET\s+\w+");
static RE_COLLATE: LazyLock<Regex> = regex!(r"(?i)\s*COLLATE\s*=?\s*\w+");
static RE_DEFAULT: LazyLock<Regex> = regex!(r"(?i)\s*DEFAULT\s+(?:'[^']*'|\d+|NULL|TRUE|FALSE)");
static RE_UNSIGNED: LazyLock<Regex> = regex!(r"(?i)\s+UNSIGNED");
static RE_CREATE_UNIQUE_INDEX: LazyLock<Regex> = regex!(r"(?i)^CREATE\s+UNIQUE\s+INDEX");
static RE_UNIQUE_KEY: LazyLock<Regex> = regex!(r"(?i)\s+UNIQUE(\s+KEY)?");
static RE_TINYINT1: LazyLock<Regex> = regex!(r"(?i)\bTINYINT\s*\(\s*1\s*\)");
static RE_INT_TYPES: LazyLock<Regex> = regex!(r"(?i)\b(BIG)?INT(EGER)?\b\s*(\(\s*\d+\s*\))?");
static RE_SMALLINT: LazyLock<Regex> = regex!(r"(?i)\b(SMALL|TINY|MEDIUM)INT\b\s*(\(\s*\d+\s*\))?");
static RE_VARCHAR: LazyLock<Regex> = regex!(r"(?i)\b(VAR)?CHAR\s*\(\s*(\d+)\s*\)");
static RE_TEXT: LazyLock<Regex> = regex!(r"(?i)\b(LONG|MEDIUM)?TEXT");
static RE_BOOL: LazyLock<Regex> = regex!(r"(?i)\bBOOL(EAN)?");
static RE_FLOAT: LazyLock<Regex> =
    regex!(r"(?i)\b(FLOAT|DOUBLE|REAL)(\s*\(\s*\d+\s*(,\s*\d+\s*)?\))?");
static RE_DECIMAL: LazyLock<Regex> =
    regex!(r"(?i)\b(DECIMAL|NUMERIC)\s*(\(\s*\d+\s*(,\s*\d+\s*)?\))?");
static RE_DATETIME: LazyLock<Regex> = regex!(r"(?i)\bDATETIME\s*(\(\s*\d+\s*\))?");
static RE_TIMESTAMP: LazyLock<Regex> = regex!(r"(?i)\bTIMESTAMP\s*(\(\s*\d+\s*\))?");
static RE_BLOB: LazyLock<Regex> = regex!(r"(?i)\b(LONG|MEDIUM|TINY)?BLOB");
static RE_BINARY16: LazyLock<Regex> = regex!(r"(?i)\bBINARY\s*\(\s*16\s*\)");
static RE_BINARY: LazyLock<Regex> = regex!(r"(?i)\b(VAR)?BINARY\s*\(\s*(\d+)\s*\)");
static RE_INLINE_PK: LazyLock<Regex> =
    regex!(r"(?i)`?(\w+)`?\s+(\w+(?:\s*\([^)]*\))?)\s+NOT\s+NULL\s+PRIMARY\s+KEY");
static RE_MULTI_SPACE: LazyLock<Regex> = regex!(r"  +");
static RE_TRAILING_COMMA: LazyLock<Regex> = regex!(r",\s*\)");

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

    sql = RE_IF_NOT_EXISTS.replace_all(&sql, "").to_string();
    sql = RE_IF_EXISTS.replace_all(&sql, "").to_string();
    sql = RE_AUTO_INCREMENT.replace_all(&sql, "").to_string();
    sql = RE_ENGINE.replace_all(&sql, "").to_string();
    sql = RE_CHARSET.replace_all(&sql, "").to_string();
    sql = RE_CHARACTER_SET.replace_all(&sql, "").to_string();
    sql = RE_COLLATE.replace_all(&sql, "").to_string();
    sql = RE_DEFAULT.replace_all(&sql, "").to_string();
    sql = RE_UNSIGNED.replace_all(&sql, "").to_string();

    if !RE_CREATE_UNIQUE_INDEX.is_match(&sql) {
        sql = RE_UNIQUE_KEY.replace_all(&sql, "").to_string();
    }

    sql = RE_TINYINT1.replace_all(&sql, "BOOL").to_string();
    sql = RE_INT_TYPES.replace_all(&sql, "INT64 ").to_string();
    sql = RE_SMALLINT.replace_all(&sql, "INT64 ").to_string();
    sql = RE_VARCHAR.replace_all(&sql, "STRING($2)").to_string();
    sql = RE_TEXT.replace_all(&sql, "STRING(MAX)").to_string();
    sql = RE_BOOL.replace_all(&sql, "BOOL").to_string();
    sql = RE_FLOAT.replace_all(&sql, "FLOAT64").to_string();
    sql = RE_DECIMAL.replace_all(&sql, "NUMERIC").to_string();
    sql = RE_DATETIME.replace_all(&sql, "TIMESTAMP ").to_string();
    sql = RE_TIMESTAMP.replace_all(&sql, "TIMESTAMP ").to_string();
    sql = RE_BLOB.replace_all(&sql, "BYTES(MAX)").to_string();
    sql = RE_BINARY16.replace_all(&sql, "UUID").to_string();
    sql = RE_BINARY.replace_all(&sql, "BYTES($2)").to_string();

    let pk_col = if let Some(caps) = RE_INLINE_PK.captures(&sql) {
        let col_name = caps.get(1).expect("capture group 1").as_str().to_string();
        let col_type = caps.get(2).expect("capture group 2").as_str().to_string();
        sql = RE_INLINE_PK
            .replace(&sql, &format!("`{}` {} NOT NULL", col_name, col_type))
            .to_string();
        Some(col_name)
    } else {
        None
    };

    sql = RE_MULTI_SPACE.replace_all(&sql, " ").to_string();
    sql = RE_TRAILING_COMMA.replace_all(&sql, ")").to_string();

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
///
/// Also holds a [`DatabaseConnection`] for executing DML (INSERT, UPDATE, DELETE)
/// within migrations via [`get_connection()`](SchemaManager::get_connection) or
/// [`execute_unprepared()`](SchemaManager::execute_unprepared).
pub struct SchemaManager {
    database_path: String,
    conn: DatabaseConnection,
}

impl SchemaManager {
    /// Create a new SchemaManager with a database connection
    pub async fn new(database_path: &str) -> Result<Self, DbErr> {
        let conn = SpannerDatabase::connect(database_path).await?;
        Ok(Self {
            database_path: database_path.to_string(),
            conn,
        })
    }

    /// Get the database path
    pub fn database_path(&self) -> &str {
        &self.database_path
    }

    /// Get a reference to the underlying database connection
    ///
    /// Use this to execute DML statements (INSERT, UPDATE, DELETE) within migrations.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    ///     // Create table first
    ///     manager.create_table_spanner(/* ... */).await?;
    ///
    ///     // Then insert seed data
    ///     let db = manager.get_connection();
    ///     db.execute_unprepared("INSERT INTO config (key, value) VALUES ('version', '1')").await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.conn
    }

    /// Execute a raw SQL statement (DML: INSERT, UPDATE, DELETE)
    ///
    /// Returns an [`ExecResult`] with the number of rows affected.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    ///     manager.create_table_spanner(/* ... */).await?;
    ///     manager.execute_unprepared(
    ///         "INSERT INTO config (key, value) VALUES ('version', '1')"
    ///     ).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        self.conn.execute_unprepared(sql).await
    }

    /// Execute multiple DDL statements
    pub async fn execute_ddl(&self, statements: Vec<String>) -> Result<(), DbErr> {
        let admin_config = if std::env::var("SPANNER_EMULATOR_HOST").is_ok() {
            AdminClientConfig::default()
        } else {
            sea_orm_spanner::ensure_tls();
            AdminClientConfig::default()
                .with_auth()
                .await
                .map_err(|e| DbErr::Custom(format!("Failed to authenticate with GCP: {}", e)))?
        };

        // Build the channel directly with a 600s (10 min) timeout instead of
        // the 30s hardcoded in AdminClient::new().
        let conn_options = ConnectionOptions {
            timeout: Some(Duration::from_secs(600)),
            connect_timeout: Some(Duration::from_secs(30)),
        };
        let conn_pool = ConnectionManager::new(
            1,
            SPANNER,
            AUDIENCE,
            &admin_config.environment,
            &conn_options,
        )
        .await
        .map_err(|e| DbErr::Custom(format!("Failed to create admin connection: {}", e)))?;
        let lro_client = OperationsClient::new(conn_pool.conn())
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to create LRO client: {}", e)))?;
        let database_admin = DatabaseAdminClient::new(conn_pool.conn(), lro_client);

        let result = database_admin
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

        let err_str = match result {
            Ok(mut op) => match op.wait(None).await {
                Ok(_) => return Ok(()),
                Err(e) => e.to_string(),
            },
            Err(e) => e.to_string(),
        };

        if err_str.contains("AlreadyExists")
            || err_str.contains("already exists")
            || err_str.contains("Duplicate name")
        {
            Ok(())
        } else {
            Err(DbErr::Custom(format!("DDL execution failed: {}", err_str)))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_unique_index_preserved() {
        let input = "CREATE UNIQUE INDEX `idx_accounts_user_id` ON `accounts` (`user_id`)";
        let result = mysql_ddl_to_spanner(input);
        assert!(
            result.contains("UNIQUE"),
            "UNIQUE must be preserved in CREATE UNIQUE INDEX, got: {}",
            result
        );
        assert_eq!(
            result,
            "CREATE UNIQUE INDEX `idx_accounts_user_id` ON `accounts` (`user_id`)"
        );
    }

    #[test]
    fn test_create_non_unique_index_unchanged() {
        let input = "CREATE INDEX `idx_accounts_email` ON `accounts` (`email`)";
        let result = mysql_ddl_to_spanner(input);
        assert_eq!(
            result,
            "CREATE INDEX `idx_accounts_email` ON `accounts` (`email`)"
        );
    }

    #[test]
    fn test_table_inline_unique_key_stripped() {
        let input =
            "CREATE TABLE `accounts` ( `id` int NOT NULL PRIMARY KEY, `email` varchar(255) NOT NULL UNIQUE KEY)";
        let result = mysql_ddl_to_spanner(input);
        assert!(
            !result.contains("UNIQUE"),
            "inline UNIQUE KEY must be stripped from table DDL, got: {}",
            result
        );
    }

    #[test]
    fn test_table_inline_unique_stripped() {
        let input =
            "CREATE TABLE `accounts` ( `id` int NOT NULL PRIMARY KEY, `email` varchar(255) NOT NULL UNIQUE)";
        let result = mysql_ddl_to_spanner(input);
        assert!(
            !result.contains("UNIQUE"),
            "inline UNIQUE must be stripped from table DDL, got: {}",
            result
        );
    }
}
