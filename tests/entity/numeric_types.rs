use {rust_decimal::Decimal, sea_orm::entity::prelude::*};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "numeric_types")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(column_type = "Decimal(Some((38, 9)))")]
    pub numeric_val: Decimal,
    #[sea_orm(column_type = "Decimal(Some((38, 9)))", nullable)]
    pub numeric_nullable: Option<Decimal>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
