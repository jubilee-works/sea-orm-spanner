use sea_orm_migration_spanner::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table_spanner(
                SpannerTableBuilder::new()
                    .table("users")
                    .string("id", Some(36), true)
                    .string("name", Some(255), true)
                    .string("email", Some(255), true)
                    .timestamp("created_at", true)
                    .primary_key(["id"]),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table_by_name("users").await
    }
}
