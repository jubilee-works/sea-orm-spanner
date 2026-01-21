mod m20220101_000001_create_users;
mod m20220102_000001_create_posts;

use sea_orm_migration_spanner::{SpannerMigrationTrait, SpannerMigratorTrait};

pub struct Migrator;

impl SpannerMigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn SpannerMigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_users::Migration),
            Box::new(m20220102_000001_create_posts::Migration),
        ]
    }
}
