use sea_orm::DbErr;
use sea_orm_migration_spanner::{SpannerMigrationTrait, SpannerSchemaManager};

pub struct Migration;

#[async_trait::async_trait]
impl SpannerMigrationTrait for Migration {
    fn name(&self) -> &str {
        "m20220102_000001_create_posts"
    }

    async fn up(&self, manager: &SpannerSchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                "CREATE TABLE posts (
                    id STRING(36) NOT NULL,
                    user_id STRING(36) NOT NULL,
                    title STRING(255) NOT NULL,
                    content STRING(MAX),
                    published BOOL NOT NULL,
                    created_at TIMESTAMP NOT NULL,
                ) PRIMARY KEY (id)",
            )
            .await
    }

    async fn down(&self, manager: &SpannerSchemaManager) -> Result<(), DbErr> {
        manager.drop_table("posts").await
    }
}
