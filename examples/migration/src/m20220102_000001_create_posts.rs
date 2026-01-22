use sea_orm_migration_spanner::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220102_000001_create_posts"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                SpannerTableBuilder::new()
                    .table("posts")
                    .string("id", Some(36), true)
                    .string("user_id", Some(36), true)
                    .string("title", Some(255), true)
                    .string("content", None, false)
                    .bool("published", true)
                    .timestamp("created_at", true)
                    .primary_key(["id"]),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table("posts").await
    }
}
