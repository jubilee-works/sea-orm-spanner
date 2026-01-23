use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "array_types")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub int64_array: Vec<i64>,
    pub int64_array_nullable: Option<Vec<i64>>,
    pub float64_array: Vec<f64>,
    pub float64_array_nullable: Option<Vec<f64>>,
    pub string_array: Vec<String>,
    pub string_array_nullable: Option<Vec<String>>,
    pub bool_array: Vec<bool>,
    pub bool_array_nullable: Option<Vec<bool>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
