use sea_orm::DbErr;
use sea_orm_migration_spanner::{SpannerMigrationTrait, SpannerSchemaManager};

pub struct Migration;

#[async_trait::async_trait]
impl SpannerMigrationTrait for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_users"
    }

    async fn up(&self, manager: &SpannerSchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                "CREATE TABLE users (
                    id STRING(36) NOT NULL,
                    name STRING(255) NOT NULL,
                    email STRING(255) NOT NULL,
                    created_at TIMESTAMP NOT NULL,
                ) PRIMARY KEY (id)",
            )
            .await
    }

    async fn down(&self, manager: &SpannerSchemaManager) -> Result<(), DbErr> {
        manager.drop_table("users").await
    }
}
