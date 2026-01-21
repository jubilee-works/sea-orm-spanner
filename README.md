# Sea-ORM Spanner

Google Cloud Spanner backend for [SeaORM](https://www.sea-ql.org/SeaORM/).

## Crates

| Crate | Description |
|-------|-------------|
| `sea-query-spanner` | SQL query builder for Spanner (converts SeaQuery to Spanner SQL) |
| `sea-orm-spanner` | SeaORM backend using ProxyDatabaseTrait |
| `sea-orm-migration-spanner` | Migration support with CLI |

## Requirements

- Rust 1.75+
- Google Cloud Spanner (or emulator for local development)

## Quick Start

### 1. Start Spanner Emulator

```bash
docker run -d -p 9010:9010 -p 9020:9020 gcr.io/cloud-spanner-emulator/emulator
export SPANNER_EMULATOR_HOST=localhost:9010
```

### 2. Add Dependencies

```toml
[dependencies]
sea-orm-spanner = { path = "path/to/sea-orm-spanner" }
sea-orm = { version = "1.1", features = ["runtime-tokio-native-tls", "macros"] }
tokio = { version = "1", features = ["full"] }
chrono = "0.4"
uuid = { version = "1", features = ["v4"] }
```

### 3. Define Entity

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

### 4. Connect and Query

```rust
use sea_orm::{EntityTrait, ActiveModelTrait, Set};
use sea_orm_spanner::SpannerDatabase;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = SpannerDatabase::connect(
        "projects/my-project/instances/my-instance/databases/my-db"
    ).await?;

    // Insert
    let user = user::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        name: Set("Alice".to_string()),
        email: Set("alice@example.com".to_string()),
        created_at: Set(chrono::Utc::now()),
    };
    let inserted = user.insert(&db).await?;

    // Query
    let users = user::Entity::find().all(&db).await?;

    // Update
    let mut active: user::ActiveModel = inserted.into();
    active.name = Set("Alice Smith".to_string());
    active.update(&db).await?;

    // Delete
    user::Entity::delete_by_id("some-id").exec(&db).await?;

    Ok(())
}
```

## Migrations

### Initialize Migration Directory

```bash
cargo run -p migration -- init --dir ./migration
```

This creates:
```
migration/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    ├── main.rs
    └── m20220101_000001_create_table.rs
```

### Generate New Migration

```bash
cargo run -p migration -- generate create_users_table
```

### Write Migration

```rust
use sea_orm::DbErr;
use sea_orm_migration_spanner::{SpannerMigrationTrait, SpannerSchemaManager};

pub struct Migration;

#[async_trait::async_trait]
impl SpannerMigrationTrait for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_users"
    }

    async fn up(&self, manager: &SpannerSchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            "CREATE TABLE users (
                id STRING(36) NOT NULL,
                name STRING(255) NOT NULL,
                email STRING(255) NOT NULL,
                created_at TIMESTAMP NOT NULL,
            ) PRIMARY KEY (id)"
        ).await
    }

    async fn down(&self, manager: &SpannerSchemaManager) -> Result<(), DbErr> {
        manager.drop_table("users").await
    }
}
```

### Run Migrations

```bash
# Set database path
export DATABASE_PATH="projects/my-project/instances/my-instance/databases/my-db"

# Check status
cargo run -p migration -- status

# Apply all pending migrations
cargo run -p migration -- up

# Apply N migrations
cargo run -p migration -- up -n 1

# Rollback last migration
cargo run -p migration -- down -n 1

# Rollback all migrations
cargo run -p migration -- reset

# Reset and reapply all
cargo run -p migration -- fresh
```

Or pass database path directly:
```bash
cargo run -p migration -- -d "projects/.../databases/..." up
```

## Testing

```bash
# Start emulator
docker run -d -p 9010:9010 -p 9020:9020 gcr.io/cloud-spanner-emulator/emulator

# Run tests
cargo test --features with-chrono,with-uuid
```

## Architecture

```
┌─────────────────────┐
│   Your Application  │
└──────────┬──────────┘
           │
┌──────────▼──────────┐
│      sea-orm        │  (ActiveRecord pattern)
└──────────┬──────────┘
           │
┌──────────▼──────────┐
│   sea-orm-spanner   │  (ProxyDatabaseTrait)
│  ┌───────────────┐  │
│  │ SQL Rewriting │  │  ? → @p1, @p2 ...
│  │ Type Convert  │  │  MySQL compat
│  └───────────────┘  │
└──────────┬──────────┘
           │
┌──────────▼──────────┐
│ google-cloud-spanner│  (gRPC client)
└──────────┬──────────┘
           │
┌──────────▼──────────┐
│   Cloud Spanner     │
└─────────────────────┘
```

### Why MySQL Backend?

SeaORM's `DbBackend` determines SQL generation behavior. Spanner doesn't support `RETURNING` clause, so:

- `DbBackend::Postgres` → Uses `INSERT ... RETURNING *` → **Fails on Spanner**
- `DbBackend::MySql` → Uses separate `SELECT` after `INSERT` → **Works on Spanner**

## Features

- `with-chrono` - DateTime support with chrono
- `with-uuid` - UUID support
- `with-json` - JSON support
- `with-rust_decimal` - Decimal support

## License

MIT OR Apache-2.0
