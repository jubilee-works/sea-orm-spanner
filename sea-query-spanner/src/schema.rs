//! Spanner DDL Schema Builder
//!
//! Provides a fluent API for building Spanner DDL statements.
//! This is separate from SeaQuery's TableBuilder because Spanner DDL
//! has significant differences from standard SQL DDL.

use crate::types::spanner_type_name;
use sea_query::ColumnType;

/// Builder for CREATE TABLE statements in Spanner DDL format
#[derive(Debug, Clone, Default)]
pub struct SpannerTableBuilder {
    table_name: String,
    columns: Vec<SpannerColumn>,
    primary_keys: Vec<String>,
    interleave_in_parent: Option<String>,
    on_delete_cascade: bool,
    row_deletion_policy: Option<String>,
}

/// Represents a column definition for Spanner
#[derive(Debug, Clone)]
pub struct SpannerColumn {
    name: String,
    column_type: String,
    not_null: bool,
    default_expr: Option<String>,
    generated_expr: Option<String>,
    stored: bool,
}

impl SpannerTableBuilder {
    /// Create a new table builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the table name
    pub fn table<S: Into<String>>(mut self, name: S) -> Self {
        self.table_name = name.into();
        self
    }

    /// Add a column with a SeaQuery ColumnType
    pub fn col<S: Into<String>>(mut self, name: S, col_type: &ColumnType, not_null: bool) -> Self {
        self.columns.push(SpannerColumn {
            name: name.into(),
            column_type: spanner_type_name(col_type),
            not_null,
            default_expr: None,
            generated_expr: None,
            stored: false,
        });
        self
    }

    /// Add a column with a raw Spanner type string
    pub fn col_raw<S: Into<String>, T: Into<String>>(
        mut self,
        name: S,
        spanner_type: T,
        not_null: bool,
    ) -> Self {
        self.columns.push(SpannerColumn {
            name: name.into(),
            column_type: spanner_type.into(),
            not_null,
            default_expr: None,
            generated_expr: None,
            stored: false,
        });
        self
    }

    /// Add a STRING column
    pub fn string<S: Into<String>>(self, name: S, max_len: Option<u32>, not_null: bool) -> Self {
        let type_str = match max_len {
            Some(len) => format!("STRING({})", len),
            None => "STRING(MAX)".to_string(),
        };
        self.col_raw(name, type_str, not_null)
    }

    /// Add an INT64 column
    pub fn int64<S: Into<String>>(self, name: S, not_null: bool) -> Self {
        self.col_raw(name, "INT64", not_null)
    }

    /// Add a FLOAT64 column
    pub fn float64<S: Into<String>>(self, name: S, not_null: bool) -> Self {
        self.col_raw(name, "FLOAT64", not_null)
    }

    /// Add a BOOL column
    pub fn bool<S: Into<String>>(self, name: S, not_null: bool) -> Self {
        self.col_raw(name, "BOOL", not_null)
    }

    /// Add a BYTES column
    pub fn bytes<S: Into<String>>(self, name: S, max_len: Option<u32>, not_null: bool) -> Self {
        let type_str = match max_len {
            Some(len) => format!("BYTES({})", len),
            None => "BYTES(MAX)".to_string(),
        };
        self.col_raw(name, type_str, not_null)
    }

    /// Add a DATE column
    pub fn date<S: Into<String>>(self, name: S, not_null: bool) -> Self {
        self.col_raw(name, "DATE", not_null)
    }

    /// Add a TIMESTAMP column
    pub fn timestamp<S: Into<String>>(self, name: S, not_null: bool) -> Self {
        self.col_raw(name, "TIMESTAMP", not_null)
    }

    /// Add a JSON column
    pub fn json<S: Into<String>>(self, name: S, not_null: bool) -> Self {
        self.col_raw(name, "JSON", not_null)
    }

    /// Add a NUMERIC column
    pub fn numeric<S: Into<String>>(self, name: S, not_null: bool) -> Self {
        self.col_raw(name, "NUMERIC", not_null)
    }

    /// Add a column with DEFAULT expression
    pub fn col_with_default<S: Into<String>, T: Into<String>, D: Into<String>>(
        mut self,
        name: S,
        spanner_type: T,
        not_null: bool,
        default_expr: D,
    ) -> Self {
        self.columns.push(SpannerColumn {
            name: name.into(),
            column_type: spanner_type.into(),
            not_null,
            default_expr: Some(default_expr.into()),
            generated_expr: None,
            stored: false,
        });
        self
    }

    /// Add a generated column
    pub fn col_generated<S: Into<String>, T: Into<String>, E: Into<String>>(
        mut self,
        name: S,
        spanner_type: T,
        expr: E,
        stored: bool,
    ) -> Self {
        self.columns.push(SpannerColumn {
            name: name.into(),
            column_type: spanner_type.into(),
            not_null: false,
            default_expr: None,
            generated_expr: Some(expr.into()),
            stored,
        });
        self
    }

    /// Set primary key columns
    pub fn primary_key<I, S>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.primary_keys = columns.into_iter().map(Into::into).collect();
        self
    }

    /// Set INTERLEAVE IN PARENT clause
    pub fn interleave_in_parent<S: Into<String>>(mut self, parent_table: S) -> Self {
        self.interleave_in_parent = Some(parent_table.into());
        self
    }

    /// Set ON DELETE CASCADE for interleaved table
    pub fn on_delete_cascade(mut self) -> Self {
        self.on_delete_cascade = true;
        self
    }

    /// Set row deletion policy (TTL)
    pub fn row_deletion_policy<S: Into<String>>(mut self, column: S, days: u32) -> Self {
        self.row_deletion_policy = Some(format!(
            "OLDER_THAN({}, INTERVAL {} DAY)",
            column.into(),
            days
        ));
        self
    }

    /// Build the CREATE TABLE DDL statement
    pub fn build(self) -> String {
        let mut ddl = format!("CREATE TABLE {} (\n", self.table_name);

        for (i, col) in self.columns.iter().enumerate() {
            if i > 0 {
                ddl.push_str(",\n");
            }
            ddl.push_str("  ");
            ddl.push_str(&col.name);
            ddl.push(' ');
            ddl.push_str(&col.column_type);

            if col.not_null {
                ddl.push_str(" NOT NULL");
            }

            if let Some(default) = &col.default_expr {
                ddl.push_str(" DEFAULT (");
                ddl.push_str(default);
                ddl.push(')');
            }

            if let Some(gen) = &col.generated_expr {
                ddl.push_str(" AS (");
                ddl.push_str(gen);
                ddl.push(')');
                if col.stored {
                    ddl.push_str(" STORED");
                }
            }
        }

        ddl.push_str("\n) PRIMARY KEY (");
        ddl.push_str(&self.primary_keys.join(", "));
        ddl.push(')');

        if let Some(parent) = &self.interleave_in_parent {
            ddl.push_str(",\n  INTERLEAVE IN PARENT ");
            ddl.push_str(parent);
            if self.on_delete_cascade {
                ddl.push_str(" ON DELETE CASCADE");
            }
        }

        if let Some(policy) = &self.row_deletion_policy {
            ddl.push_str(",\n  ROW DELETION POLICY (");
            ddl.push_str(policy);
            ddl.push(')');
        }

        ddl
    }
}

/// Builder for CREATE INDEX statements in Spanner DDL format
#[derive(Debug, Clone, Default)]
pub struct SpannerIndexBuilder {
    index_name: String,
    table_name: String,
    columns: Vec<(String, Option<bool>)>, // (column_name, is_desc)
    unique: bool,
    null_filtered: bool,
    storing: Vec<String>,
    interleave_in: Option<String>,
}

impl SpannerIndexBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set index name
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.index_name = name.into();
        self
    }

    /// Set table name
    pub fn table<S: Into<String>>(mut self, name: S) -> Self {
        self.table_name = name.into();
        self
    }

    /// Add a column to the index
    pub fn col<S: Into<String>>(mut self, name: S) -> Self {
        self.columns.push((name.into(), None));
        self
    }

    /// Add a column with ASC order
    pub fn col_asc<S: Into<String>>(mut self, name: S) -> Self {
        self.columns.push((name.into(), Some(false)));
        self
    }

    /// Add a column with DESC order
    pub fn col_desc<S: Into<String>>(mut self, name: S) -> Self {
        self.columns.push((name.into(), Some(true)));
        self
    }

    /// Make this a unique index
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Make this a null-filtered index
    pub fn null_filtered(mut self) -> Self {
        self.null_filtered = true;
        self
    }

    /// Add STORING columns
    pub fn storing<I, S>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.storing = columns.into_iter().map(Into::into).collect();
        self
    }

    /// Set INTERLEAVE IN clause
    pub fn interleave_in<S: Into<String>>(mut self, table: S) -> Self {
        self.interleave_in = Some(table.into());
        self
    }

    /// Build the CREATE INDEX DDL statement
    pub fn build(self) -> String {
        let mut ddl = String::new();
        ddl.push_str("CREATE ");

        if self.unique {
            ddl.push_str("UNIQUE ");
        }
        if self.null_filtered {
            ddl.push_str("NULL_FILTERED ");
        }

        ddl.push_str("INDEX ");
        ddl.push_str(&self.index_name);
        ddl.push_str(" ON ");
        ddl.push_str(&self.table_name);
        ddl.push_str(" (");

        for (i, (col, order)) in self.columns.iter().enumerate() {
            if i > 0 {
                ddl.push_str(", ");
            }
            ddl.push_str(col);
            if let Some(is_desc) = order {
                ddl.push_str(if *is_desc { " DESC" } else { " ASC" });
            }
        }

        ddl.push(')');

        if !self.storing.is_empty() {
            ddl.push_str(" STORING (");
            ddl.push_str(&self.storing.join(", "));
            ddl.push(')');
        }

        if let Some(table) = &self.interleave_in {
            ddl.push_str(", INTERLEAVE IN ");
            ddl.push_str(table);
        }

        ddl
    }
}

/// Builder for ALTER TABLE statements in Spanner DDL format
#[derive(Debug, Clone)]
pub enum SpannerAlterTable {
    AddColumn {
        table: String,
        column: SpannerColumn,
    },
    DropColumn {
        table: String,
        column: String,
    },
    AlterColumn {
        table: String,
        column: String,
        new_type: Option<String>,
        set_not_null: Option<bool>,
        set_default: Option<String>,
        drop_default: bool,
    },
    AddForeignKey {
        table: String,
        constraint_name: String,
        columns: Vec<String>,
        ref_table: String,
        ref_columns: Vec<String>,
        on_delete: Option<String>,
    },
    DropConstraint {
        table: String,
        constraint_name: String,
    },
}

impl SpannerAlterTable {
    pub fn add_column<T: Into<String>, N: Into<String>, S: Into<String>>(
        table: T,
        name: N,
        spanner_type: S,
        not_null: bool,
    ) -> Self {
        Self::AddColumn {
            table: table.into(),
            column: SpannerColumn {
                name: name.into(),
                column_type: spanner_type.into(),
                not_null,
                default_expr: None,
                generated_expr: None,
                stored: false,
            },
        }
    }

    pub fn drop_column<T: Into<String>, N: Into<String>>(table: T, column: N) -> Self {
        Self::DropColumn {
            table: table.into(),
            column: column.into(),
        }
    }

    pub fn build(self) -> String {
        match self {
            Self::AddColumn { table, column } => {
                let mut ddl = format!(
                    "ALTER TABLE {} ADD COLUMN {} {}",
                    table, column.name, column.column_type
                );
                if column.not_null {
                    ddl.push_str(" NOT NULL");
                }
                if let Some(default) = column.default_expr {
                    ddl.push_str(" DEFAULT (");
                    ddl.push_str(&default);
                    ddl.push(')');
                }
                ddl
            }
            Self::DropColumn { table, column } => {
                format!("ALTER TABLE {} DROP COLUMN {}", table, column)
            }
            Self::AlterColumn {
                table,
                column,
                new_type,
                set_not_null,
                set_default,
                drop_default,
            } => {
                let mut ddl = format!("ALTER TABLE {} ALTER COLUMN {}", table, column);
                if let Some(t) = new_type {
                    ddl.push(' ');
                    ddl.push_str(&t);
                }
                if let Some(nn) = set_not_null {
                    if nn {
                        ddl.push_str(" NOT NULL");
                    }
                }
                if let Some(def) = set_default {
                    ddl.push_str(" DEFAULT (");
                    ddl.push_str(&def);
                    ddl.push(')');
                }
                if drop_default {
                    ddl.push_str(" DROP DEFAULT");
                }
                ddl
            }
            Self::AddForeignKey {
                table,
                constraint_name,
                columns,
                ref_table,
                ref_columns,
                on_delete,
            } => {
                let mut ddl = format!(
                    "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({})",
                    table,
                    constraint_name,
                    columns.join(", "),
                    ref_table,
                    ref_columns.join(", ")
                );
                if let Some(action) = on_delete {
                    ddl.push_str(" ON DELETE ");
                    ddl.push_str(&action);
                }
                ddl
            }
            Self::DropConstraint {
                table,
                constraint_name,
            } => {
                format!("ALTER TABLE {} DROP CONSTRAINT {}", table, constraint_name)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_table_basic() {
        let ddl = SpannerTableBuilder::new()
            .table("users")
            .string("id", Some(36), true)
            .string("name", None, true)
            .string("email", None, false)
            .timestamp("created_at", true)
            .primary_key(["id"])
            .build();

        assert_eq!(
            ddl,
            "CREATE TABLE users (\n  id STRING(36) NOT NULL,\n  name STRING(MAX) NOT NULL,\n  email STRING(MAX),\n  created_at TIMESTAMP NOT NULL\n) PRIMARY KEY (id)"
        );
    }

    #[test]
    fn test_create_table_interleaved() {
        let ddl = SpannerTableBuilder::new()
            .table("posts")
            .string("user_id", Some(36), true)
            .string("post_id", Some(36), true)
            .string("content", None, true)
            .primary_key(["user_id", "post_id"])
            .interleave_in_parent("users")
            .on_delete_cascade()
            .build();

        assert!(ddl.contains("INTERLEAVE IN PARENT users ON DELETE CASCADE"));
    }

    #[test]
    fn test_create_index() {
        let ddl = SpannerIndexBuilder::new()
            .name("idx_users_email")
            .table("users")
            .col("email")
            .unique()
            .build();

        assert_eq!(ddl, "CREATE UNIQUE INDEX idx_users_email ON users (email)");
    }

    #[test]
    fn test_create_index_with_storing() {
        let ddl = SpannerIndexBuilder::new()
            .name("idx_users_name")
            .table("users")
            .col("name")
            .storing(["email", "created_at"])
            .build();

        assert_eq!(
            ddl,
            "CREATE INDEX idx_users_name ON users (name) STORING (email, created_at)"
        );
    }

    #[test]
    fn test_alter_table_add_column() {
        let ddl = SpannerAlterTable::add_column("users", "age", "INT64", false).build();
        assert_eq!(ddl, "ALTER TABLE users ADD COLUMN age INT64");
    }

    #[test]
    fn test_alter_table_drop_column() {
        let ddl = SpannerAlterTable::drop_column("users", "age").build();
        assert_eq!(ddl, "ALTER TABLE users DROP COLUMN age");
    }
}
