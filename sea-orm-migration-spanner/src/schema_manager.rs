use google_cloud_googleapis::spanner::admin::database::v1::UpdateDatabaseDdlRequest;
use google_cloud_spanner::admin::database::database_admin_client::DatabaseAdminClient;
use sea_orm::DbErr;
use sea_query_spanner::{SpannerAlterTable, SpannerIndexBuilder, SpannerTableBuilder};

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
        let db_client = DatabaseAdminClient::default()
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to create admin client: {}", e)))?;

        let result = db_client
            .update_database_ddl(
                UpdateDatabaseDdlRequest {
                    database: self.database_path.clone(),
                    statements,
                    operation_id: "".to_string(),
                },
                None,
                None,
            )
            .await;

        match result {
            Ok(mut op) => {
                op.wait(None, None)
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

    /// Create a table using the builder
    pub async fn create_table(&self, builder: SpannerTableBuilder) -> Result<(), DbErr> {
        self.execute_ddl(vec![builder.build()]).await
    }

    /// Drop a table
    pub async fn drop_table(&self, table_name: &str) -> Result<(), DbErr> {
        self.execute_ddl(vec![format!("DROP TABLE {}", table_name)])
            .await
    }

    /// Drop a table if it exists (ignores errors if table doesn't exist)
    pub async fn drop_table_if_exists(&self, table_name: &str) -> Result<(), DbErr> {
        let result = self.drop_table(table_name).await;
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

    /// Create an index using raw DDL
    pub async fn create_index_raw(&self, ddl: &str) -> Result<(), DbErr> {
        self.execute_ddl(vec![ddl.to_string()]).await
    }

    /// Create an index using the builder
    pub async fn create_index(&self, builder: SpannerIndexBuilder) -> Result<(), DbErr> {
        self.execute_ddl(vec![builder.build()]).await
    }

    /// Drop an index
    pub async fn drop_index(&self, index_name: &str) -> Result<(), DbErr> {
        self.execute_ddl(vec![format!("DROP INDEX {}", index_name)])
            .await
    }

    /// Drop an index if it exists
    pub async fn drop_index_if_exists(&self, index_name: &str) -> Result<(), DbErr> {
        let result = self.drop_index(index_name).await;
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
    // Alter Table Operations
    // ========================================

    /// Alter a table using the builder
    pub async fn alter_table(&self, alter: SpannerAlterTable) -> Result<(), DbErr> {
        self.execute_ddl(vec![alter.build()]).await
    }

    /// Add a column to a table
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

    /// Drop a column from a table
    pub async fn drop_column(&self, table: &str, column_name: &str) -> Result<(), DbErr> {
        let alter = SpannerAlterTable::drop_column(table, column_name);
        self.execute_ddl(vec![alter.build()]).await
    }
}
