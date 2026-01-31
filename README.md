# sea-orm-spanner

Google Cloud Spanner backend for [SeaORM](https://www.sea-ql.org/SeaORM/).

## Sub-crates

| Crate | Description |
|-------|-------------|
| `sea-query-spanner` | SQL query builder for Spanner (converts SeaQuery to Spanner SQL) |
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
    pub created_at: DateTime,  // Use DateTime (NaiveDateTime), not DateTimeUtc
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
        created_at: Set(chrono::Utc::now().naive_utc()),
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
use sea_orm_migration_spanner::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_users"
    }
}

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
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
        manager.drop_table("users").await
    }
}
```

You can also use raw DDL if needed:

```rust
manager.create_table_raw(
    "CREATE TABLE users (
        id STRING(36) NOT NULL,
        name STRING(255) NOT NULL,
    ) PRIMARY KEY (id)"
).await
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
- `with-rust_decimal` - NUMERIC/Decimal support
- `with-array` - ARRAY type support (INT64, FLOAT64, STRING, BOOL arrays)

## Known Limitations

### Type Mapping

Spanner has a limited set of native types compared to other databases. This library maps SeaORM types to Spanner types with the following considerations:

#### Integer Types

Spanner only has `INT64`. When reading values:
- Values in i32 range (-2147483648 to 2147483647) are returned as `i32`
- Values outside i32 range are returned as `i64`

**Recommendation**: Use `i32` for entity fields that will always contain small values. Use `i64` with values outside i32 range to ensure correct type mapping.

```rust
pub struct Model {
    pub small_count: i32,      // For values that fit in i32
    pub large_id: i64,         // Use values > i32::MAX or < i32::MIN
}
```

#### Float Types

Spanner only has `FLOAT64`. Use `f64` in your entities, not `f32`.

```rust
pub struct Model {
    pub price: f64,            // Correct: use f64
    // pub price: f32,         // Avoid: will cause type mismatch
}
```

#### TIMESTAMP Type

Spanner TIMESTAMP columns must use `DateTime` (`chrono::NaiveDateTime`) in entity definitions, **not** `DateTimeUtc` (`chrono::DateTime<chrono::Utc>`).

```rust
pub struct Model {
    pub created_at: DateTime,           // Correct: NaiveDateTime
    // pub created_at: DateTimeUtc,     // Wrong: will return None on read
}
```

Spanner stores all timestamps in UTC. When you need timezone-aware datetime, convert after reading:

```rust
let utc_time = model.created_at.and_utc();  // NaiveDateTime -> DateTime<Utc>
```

#### BYTES vs STRING

Both BYTES and STRING columns are transmitted as strings (BYTES are base64-encoded). The library uses heuristics to distinguish them:
- Strings containing base64 special characters (`+`, `/`, `=`) that decode to non-UTF8 or null bytes are treated as BYTES
- Empty strings cannot be distinguished and are treated as STRING

**Recommendation**: Avoid storing empty byte arrays. Use at least one byte (e.g., `vec![0]`) for BYTES columns that need to represent "empty".

#### JSON Primitives

JSON columns containing simple numeric values (e.g., `42`, `3.14`) cannot be distinguished from INT64/FLOAT64 columns at read time. This limitation affects JSON columns storing primitive numbers.

**Recommendation**: Wrap JSON primitives in objects or arrays:

```rust
// Instead of:
json_val: Set(json!(42))

// Use:
json_val: Set(json!({"value": 42}))
```

#### ARRAY Types

Spanner ARRAY types are supported for the following element types:
- `ARRAY<INT64>` → `Vec<i64>`
- `ARRAY<FLOAT64>` → `Vec<f64>`
- `ARRAY<STRING>` → `Vec<String>`
- `ARRAY<BOOL>` → `Vec<bool>`

**Limitation**: Empty arrays cannot be reliably read back from Spanner due to SDK limitations. The Spanner SDK returns empty arrays without type information, making it impossible to determine the correct element type. Always store at least one element in arrays, or use nullable arrays with `NULL` instead of empty arrays.

Example entity:

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "my_table")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub tags: Vec<String>,              // ARRAY<STRING(MAX)>
    pub scores: Vec<i64>,               // ARRAY<INT64>
    pub optional_flags: Option<Vec<bool>>, // ARRAY<BOOL> nullable
}
```

#### NUMERIC Type

Spanner NUMERIC type is supported via `rust_decimal::Decimal`. NUMERIC provides 38 digits of precision with 9 decimal places.

**Limitation**: Due to Spanner SDK limitations with type detection, avoid using NUMERIC with special values like zero in the same table as STRING columns. The SDK may misinterpret types when reading null or zero values.

Example entity:

```rust
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "products")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(column_type = "Decimal(Some((38, 9)))")]
    pub price: Decimal,
    #[sea_orm(column_type = "Decimal(Some((38, 9)))", nullable)]
    pub discount: Option<Decimal>,
}
```

## License

MIT OR Apache-2.0
