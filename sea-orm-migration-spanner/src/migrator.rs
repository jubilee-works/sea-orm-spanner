use crate::schema_manager::SchemaManager;
use sea_orm::sea_query::{Alias, Order, Query};
use sea_orm::{ActiveValue, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter};
use sea_orm_spanner::SpannerDatabase;
use std::collections::HashSet;
use std::time::SystemTime;
use tracing::info;

mod seaql_migrations {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "seaql_migrations")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub version: String,
        pub applied_at: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

const MIGRATIONS_TABLE_DDL: &str = "CREATE TABLE seaql_migrations (
    version STRING(255) NOT NULL,
    applied_at INT64 NOT NULL,
) PRIMARY KEY (version)";

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MigrationStatus {
    Pending,
    Applied,
}

pub struct Migration {
    migration: Box<dyn MigrationTrait>,
    status: MigrationStatus,
}

impl Migration {
    pub fn name(&self) -> &str {
        self.migration.name()
    }

    pub fn status(&self) -> MigrationStatus {
        self.status
    }
}

pub use sea_orm_migration::MigrationName;

#[async_trait::async_trait]
pub trait MigrationTrait: MigrationName + Send + Sync {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr>;

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "Rollback not implemented for this migration".to_owned(),
        ))
    }
}

#[async_trait::async_trait]
pub trait MigratorTrait: Send {
    fn migrations() -> Vec<Box<dyn MigrationTrait>>;

    fn get_migration_files() -> Vec<Migration> {
        Self::migrations()
            .into_iter()
            .map(|migration| Migration {
                migration,
                status: MigrationStatus::Pending,
            })
            .collect()
    }

    async fn install(database_path: &str) -> Result<(), DbErr> {
        let schema_manager = SchemaManager::new(database_path);
        schema_manager.create_table_raw(MIGRATIONS_TABLE_DDL).await
    }

    async fn get_applied_versions(database_path: &str) -> Result<HashSet<String>, DbErr> {
        let db = SpannerDatabase::connect(database_path).await?;

        let stmt = Query::select()
            .from(Alias::new("seaql_migrations"))
            .column(Alias::new("version"))
            .order_by(Alias::new("version"), Order::Asc)
            .to_owned();

        let results = db.query_all(&stmt).await?;

        let mut versions = HashSet::new();
        for row in results {
            let version: String = row.try_get("", "version")?;
            versions.insert(version);
        }

        Ok(versions)
    }

    async fn get_migrations_with_status(database_path: &str) -> Result<Vec<Migration>, DbErr> {
        Self::install(database_path).await?;

        let mut migrations = Self::get_migration_files();
        let applied = Self::get_applied_versions(database_path).await?;

        for migration in migrations.iter_mut() {
            if applied.contains(migration.name()) {
                migration.status = MigrationStatus::Applied;
            }
        }

        Ok(migrations)
    }

    async fn status(database_path: &str) -> Result<(), DbErr> {
        info!("Checking migration status");

        let migrations = Self::get_migrations_with_status(database_path).await?;

        for migration in migrations {
            let status = match migration.status {
                MigrationStatus::Pending => "Pending",
                MigrationStatus::Applied => "Applied",
            };
            info!("Migration '{}'... {}", migration.name(), status);
            println!("Migration '{}'... {}", migration.name(), status);
        }

        Ok(())
    }

    async fn up(database_path: &str, steps: Option<u32>) -> Result<(), DbErr> {
        let migrations = Self::get_migrations_with_status(database_path).await?;
        let pending: Vec<_> = migrations
            .into_iter()
            .filter(|m| m.status == MigrationStatus::Pending)
            .collect();

        if pending.is_empty() {
            info!("No pending migrations");
            println!("No pending migrations");
            return Ok(());
        }

        let schema_manager = SchemaManager::new(database_path);
        let db = SpannerDatabase::connect(database_path).await?;

        let to_apply: Vec<_> = match steps {
            Some(n) => pending.into_iter().take(n as usize).collect(),
            None => pending,
        };

        info!("Applying {} migrations", to_apply.len());
        println!("Applying {} migrations", to_apply.len());

        for migration in to_apply {
            info!("Applying migration '{}'", migration.name());
            println!("Applying migration '{}'", migration.name());

            migration.migration.up(&schema_manager).await?;

            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("SystemTime before UNIX EPOCH!")
                .as_secs() as i64;

            seaql_migrations::Entity::insert(seaql_migrations::ActiveModel {
                version: ActiveValue::Set(migration.name().to_string()),
                applied_at: ActiveValue::Set(now),
            })
            .exec(&db)
            .await?;

            info!("Migration '{}' has been applied", migration.name());
            println!("Migration '{}' has been applied", migration.name());
        }

        Ok(())
    }

    async fn down(database_path: &str, steps: Option<u32>) -> Result<(), DbErr> {
        let migrations = Self::get_migrations_with_status(database_path).await?;
        let mut applied: Vec<_> = migrations
            .into_iter()
            .filter(|m| m.status == MigrationStatus::Applied)
            .collect();

        applied.reverse();

        if applied.is_empty() {
            info!("No applied migrations to rollback");
            println!("No applied migrations to rollback");
            return Ok(());
        }

        let schema_manager = SchemaManager::new(database_path);
        let db = SpannerDatabase::connect(database_path).await?;

        let to_rollback: Vec<_> = match steps {
            Some(n) => applied.into_iter().take(n as usize).collect(),
            None => applied,
        };

        info!("Rolling back {} migrations", to_rollback.len());
        println!("Rolling back {} migrations", to_rollback.len());

        for migration in to_rollback {
            info!("Rolling back migration '{}'", migration.name());
            println!("Rolling back migration '{}'", migration.name());

            migration.migration.down(&schema_manager).await?;

            seaql_migrations::Entity::delete_many()
                .filter(seaql_migrations::Column::Version.eq(migration.name()))
                .exec(&db)
                .await?;

            info!("Migration '{}' has been rolled back", migration.name());
            println!("Migration '{}' has been rolled back", migration.name());
        }

        Ok(())
    }

    async fn fresh(database_path: &str) -> Result<(), DbErr> {
        info!("Dropping all tables and reapplying migrations");
        println!("Dropping all tables and reapplying migrations");

        let schema_manager = SchemaManager::new(database_path);

        let migrations = Self::get_migration_files();
        for migration in migrations.iter().rev() {
            let _ = migration.migration.down(&schema_manager).await;
        }

        let _ = schema_manager.drop_table_by_name("seaql_migrations").await;

        Self::up(database_path, None).await
    }

    async fn reset(database_path: &str) -> Result<(), DbErr> {
        Self::down(database_path, None).await?;

        let schema_manager = SchemaManager::new(database_path);
        schema_manager.drop_table_by_name("seaql_migrations").await
    }
}
