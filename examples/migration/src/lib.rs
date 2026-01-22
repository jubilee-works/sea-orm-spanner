mod m20220101_000001_create_users;
mod m20220102_000001_create_posts;

use sea_orm_migration_spanner::prelude::*;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_users::Migration),
            Box::new(m20220102_000001_create_posts::Migration),
        ]
    }
}
