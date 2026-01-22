use sea_orm_migration_spanner::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Username,
    Password,
}

#[derive(DeriveIden)]
enum Category {
    Table,
    Name,
}

#[derive(DeriveIden)]
enum Product {
    Table,
    Id,
    Title,
    Price,
    Category,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::Username).string().not_null())
                    .col(ColumnDef::new(Users::Password).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Category::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Category::Name)
                            .string()
                            .unique_key()
                            .not_null()
                            .primary_key(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Product::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Product::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Product::Title).string().not_null())
                    .col(ColumnDef::new(Product::Price).integer().not_null())
                    .col(ColumnDef::new(Product::Category).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-product-category")
                            .from(Product::Table, Product::Category)
                            .to(Category::Table, Category::Name),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).if_exists().to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Product::Table).if_exists().to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Category::Table).if_exists().to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Post {
    Table,
    Id,
    Title,
    Text,
}
