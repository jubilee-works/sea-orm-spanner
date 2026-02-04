use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "all_types")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,

    pub string_val: String,
    pub string_nullable: Option<String>,

    pub int64_val: i64,
    pub int64_nullable: Option<i64>,
    pub int32_val: i64,
    pub int32_nullable: Option<i64>,

    pub float64_val: f64,
    pub float64_nullable: Option<f64>,
    pub float32_val: f64,
    pub float32_nullable: Option<f64>,

    pub bool_val: bool,
    pub bool_nullable: Option<bool>,

    pub bytes_val: Vec<u8>,
    pub bytes_nullable: Option<Vec<u8>>,

    pub timestamp_val: DateTime,
    pub timestamp_nullable: Option<DateTime>,

    pub date_val: Date,
    pub date_nullable: Option<Date>,

    #[sea_orm(column_type = "Json")]
    pub json_val: Json,
    #[sea_orm(column_type = "Json", nullable)]
    pub json_nullable: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
