use google_cloud_googleapis::spanner::admin::database::v1::UpdateDatabaseDdlRequest;
use google_cloud_spanner::admin::database::database_admin_client::DatabaseAdminClient;
use sea_orm::DbErr;

pub struct SpannerSchemaManager {
    database_path: String,
}

impl SpannerSchemaManager {
    pub fn new(database_path: &str) -> Self {
        Self {
            database_path: database_path.to_string(),
        }
    }

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

    pub async fn create_table(&self, ddl: &str) -> Result<(), DbErr> {
        self.execute_ddl(vec![ddl.to_string()]).await
    }

    pub async fn drop_table(&self, table_name: &str) -> Result<(), DbErr> {
        self.execute_ddl(vec![format!("DROP TABLE {}", table_name)])
            .await
    }

    pub async fn create_index(&self, ddl: &str) -> Result<(), DbErr> {
        self.execute_ddl(vec![ddl.to_string()]).await
    }

    pub async fn drop_index(&self, index_name: &str) -> Result<(), DbErr> {
        self.execute_ddl(vec![format!("DROP INDEX {}", index_name)])
            .await
    }
}
