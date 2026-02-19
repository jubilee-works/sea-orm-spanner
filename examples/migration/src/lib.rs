mod unimplemented_create_table;

use sea_orm_migration_spanner::prelude::*;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(unimplemented_create_table::Migration)]
    }
}
