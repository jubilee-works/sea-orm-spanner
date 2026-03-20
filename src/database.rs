use {
    crate::{error::SpannerDbErr, proxy::SpannerProxy},
    gcloud_gax::conn::Environment,
    gcloud_googleapis::spanner::admin::{
        database::v1::{CreateDatabaseRequest, DatabaseDialect as GrpcDatabaseDialect},
        instance::v1::{CreateInstanceRequest, Instance},
    },
    gcloud_spanner::{
        admin::{client::Client as AdminClient, AdminClientConfig},
        client::{Client, ClientConfig},
    },
    sea_orm::{Database, DatabaseConnection, DbErr},
    std::sync::Arc,
};

/// Database dialect for Spanner
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DatabaseDialect {
    #[default]
    GoogleStandardSql,
    PostgreSql,
}

impl From<DatabaseDialect> for i32 {
    fn from(dialect: DatabaseDialect) -> Self {
        match dialect {
            DatabaseDialect::GoogleStandardSql => GrpcDatabaseDialect::GoogleStandardSql.into(),
            DatabaseDialect::PostgreSql => GrpcDatabaseDialect::Postgresql.into(),
        }
    }
}

/// Configuration for Spanner instance creation
#[derive(Debug, Clone, Default)]
pub struct InstanceConfig {
    /// Display name for the instance
    pub display_name: Option<String>,
    /// Instance configuration (e.g., "regional-us-central1")
    /// For emulator, this can be empty
    pub config: Option<String>,
    /// Number of nodes (for production instances)
    pub node_count: Option<i32>,
    /// Processing units (alternative to node_count)
    pub processing_units: Option<i32>,
}

/// Options for creating Spanner instance and database
#[derive(Debug, Clone)]
pub struct CreateOptions {
    /// Create instance if it doesn't exist (default: false)
    ///
    /// **Warning**: Instance creation can take several minutes and requires
    /// appropriate IAM permissions. Usually only needed for emulator/testing.
    pub create_instance_if_not_exists: bool,

    /// Create database if it doesn't exist (default: true)
    pub create_database_if_not_exists: bool,

    /// Configuration for instance creation
    pub instance_config: InstanceConfig,

    /// Database dialect (default: GoogleStandardSql)
    pub database_dialect: DatabaseDialect,
}

impl Default for CreateOptions {
    fn default() -> Self {
        Self {
            create_instance_if_not_exists: false,
            create_database_if_not_exists: true,
            instance_config: InstanceConfig::default(),
            database_dialect: DatabaseDialect::default(),
        }
    }
}

impl CreateOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_instance_creation(mut self) -> Self {
        self.create_instance_if_not_exists = true;
        self
    }

    pub fn with_dialect(mut self, dialect: DatabaseDialect) -> Self {
        self.database_dialect = dialect;
        self
    }

    pub fn with_instance_config(mut self, config: InstanceConfig) -> Self {
        self.instance_config = config;
        self
    }
}

/// Parsed components of a Spanner database path
#[derive(Debug, Clone)]
pub struct DatabasePath {
    pub project: String,
    pub instance: String,
    pub database: String,
}

impl DatabasePath {
    /// Parse a database path string into components
    ///
    /// Expected format: `projects/{project}/instances/{instance}/databases/{database}`
    pub fn parse(path: &str) -> Result<Self, DbErr> {
        let parts: Vec<&str> = path.split('/').collect();

        if parts.len() != 6
            || parts[0] != "projects"
            || parts[2] != "instances"
            || parts[4] != "databases"
        {
            return Err(DbErr::Custom(format!(
                "Invalid database path format. Expected: projects/{{project}}/instances/{{instance}}/databases/{{database}}, got: {}",
                path
            )));
        }

        Ok(Self {
            project: parts[1].to_string(),
            instance: parts[3].to_string(),
            database: parts[5].to_string(),
        })
    }

    /// Get the full database path
    pub fn full_path(&self) -> String {
        format!(
            "projects/{}/instances/{}/databases/{}",
            self.project, self.instance, self.database
        )
    }

    /// Get the project path
    pub fn project_path(&self) -> String {
        format!("projects/{}", self.project)
    }

    /// Get the instance path
    pub fn instance_path(&self) -> String {
        format!("projects/{}/instances/{}", self.project, self.instance)
    }
}

/// Install the rustls crypto provider for GCP TLS connections.
///
/// This is called automatically when connecting to GCP (non-emulator).
/// Safe to call multiple times — subsequent calls are no-ops.
pub fn ensure_tls() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
}

pub struct SpannerDatabase;

impl SpannerDatabase {
    /// Connect to an existing Spanner database
    ///
    /// Automatically detects the environment:
    /// - If `SPANNER_EMULATOR_HOST` is set, connects without authentication
    /// - Otherwise, uses Application Default Credentials (ADC) for authentication
    ///
    /// ADC discovers credentials from (in order):
    /// 1. `GOOGLE_APPLICATION_CREDENTIALS` environment variable (service account JSON)
    /// 2. `gcloud auth application-default login` (local development)
    /// 3. GCE/GKE metadata server (when running on Google Cloud)
    pub async fn connect(database: &str) -> Result<DatabaseConnection, DbErr> {
        let config = if std::env::var("SPANNER_EMULATOR_HOST").is_ok() {
            ClientConfig::default()
        } else {
            ensure_tls();
            ClientConfig::default().with_auth().await.map_err(|e| {
                SpannerDbErr::Connection(format!("Failed to authenticate with GCP: {}", e))
            })?
        };
        Self::connect_with_config(database, config).await
    }

    /// Connect to an existing Spanner database with custom configuration
    pub async fn connect_with_config(
        database: &str,
        config: ClientConfig,
    ) -> Result<DatabaseConnection, DbErr> {
        let client = Client::new(database, config)
            .await
            .map_err(|e| SpannerDbErr::Connection(e.to_string()))?;

        let proxy = SpannerProxy::new(Arc::new(client));
        Database::connect_proxy(sea_orm::DbBackend::MySql, Arc::new(Box::new(proxy))).await
    }

    /// Connect to Spanner using the local emulator
    pub async fn connect_with_emulator(database: &str) -> Result<DatabaseConnection, DbErr> {
        Self::connect_with_emulator_host(database, "localhost:9010").await
    }

    /// Connect to Spanner using a custom emulator host
    pub async fn connect_with_emulator_host(
        database: &str,
        emulator_host: &str,
    ) -> Result<DatabaseConnection, DbErr> {
        let config = ClientConfig {
            environment: Environment::Emulator(emulator_host.to_string()),
            ..Default::default()
        };
        Self::connect_with_config(database, config).await
    }

    /// Connect to Spanner emulator, creating instance and/or database if they don't exist
    ///
    /// **Note**: This function only works with the Spanner emulator (localhost:9010).
    /// It will fail if `SPANNER_EMULATOR_HOST` is not set.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use sea_orm_spanner::{SpannerDatabase, CreateOptions};
    ///
    /// // Create database if not exists (default behavior)
    /// let db = SpannerDatabase::connect_or_create_with_emulator(
    ///     "projects/test/instances/test/databases/test",
    ///     CreateOptions::default(),
    /// ).await?;
    ///
    /// // Also create instance if not exists
    /// let db = SpannerDatabase::connect_or_create_with_emulator(
    ///     "projects/test/instances/test/databases/test",
    ///     CreateOptions::new().with_instance_creation(),
    /// ).await?;
    /// ```
    pub async fn connect_or_create_with_emulator(
        database: &str,
        options: CreateOptions,
    ) -> Result<DatabaseConnection, DbErr> {
        Self::connect_or_create_with_emulator_host(database, "localhost:9010", options).await
    }

    /// Connect to Spanner emulator at custom host, creating instance/database if needed
    ///
    /// **Note**: This function only works with the Spanner emulator.
    pub async fn connect_or_create_with_emulator_host(
        database: &str,
        emulator_host: &str,
        options: CreateOptions,
    ) -> Result<DatabaseConnection, DbErr> {
        let path = DatabasePath::parse(database)?;

        if options.create_instance_if_not_exists {
            ensure_instance(&path, &options.instance_config, emulator_host).await?;
        }

        if options.create_database_if_not_exists {
            ensure_database(&path, options.database_dialect, emulator_host).await?;
        }

        let config = ClientConfig {
            environment: Environment::Emulator(emulator_host.to_string()),
            ..Default::default()
        };
        Self::connect_with_config(database, config).await
    }
}

pub async fn ensure_instance(
    path: &DatabasePath,
    config: &InstanceConfig,
    emulator_host: &str,
) -> Result<bool, DbErr> {
    let admin_config = AdminClientConfig {
        environment: Environment::Emulator(emulator_host.to_string()),
        ..Default::default()
    };
    let admin_client = AdminClient::new(admin_config)
        .await
        .map_err(|e| SpannerDbErr::Connection(format!("Failed to create admin client: {}", e)))?;

    let display_name = config
        .display_name
        .clone()
        .unwrap_or_else(|| format!("{} Instance", path.instance));

    let instance_config = config.config.clone().unwrap_or_default();

    let mut instance = Instance {
        name: path.instance_path(),
        config: instance_config,
        display_name,
        ..Default::default()
    };

    if let Some(node_count) = config.node_count {
        instance.node_count = node_count;
    }
    if let Some(processing_units) = config.processing_units {
        instance.processing_units = processing_units;
    }

    let result = admin_client
        .instance()
        .create_instance(
            CreateInstanceRequest {
                parent: path.project_path(),
                instance_id: path.instance.clone(),
                instance: Some(instance),
            },
            None,
        )
        .await;

    match result {
        Ok(mut op) => {
            op.wait(None).await.map_err(|e| {
                SpannerDbErr::Connection(format!("Instance creation failed: {}", e))
            })?;
            Ok(true)
        }
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("AlreadyExists") || err_str.contains("already exists") {
                Ok(false)
            } else {
                Err(SpannerDbErr::Connection(format!("Failed to create instance: {}", e)).into())
            }
        }
    }
}

pub async fn ensure_database(
    path: &DatabasePath,
    dialect: DatabaseDialect,
    emulator_host: &str,
) -> Result<bool, DbErr> {
    let admin_config = AdminClientConfig {
        environment: Environment::Emulator(emulator_host.to_string()),
        ..Default::default()
    };
    let admin_client = AdminClient::new(admin_config)
        .await
        .map_err(|e| SpannerDbErr::Connection(format!("Failed to create admin client: {}", e)))?;

    let result = admin_client
        .database()
        .create_database(
            CreateDatabaseRequest {
                parent: path.instance_path(),
                create_statement: format!("CREATE DATABASE `{}`", path.database),
                extra_statements: vec![],
                encryption_config: None,
                database_dialect: dialect.into(),
                proto_descriptors: vec![],
            },
            None,
        )
        .await;

    match result {
        Ok(mut op) => {
            op.wait(None).await.map_err(|e| {
                SpannerDbErr::Connection(format!("Database creation failed: {}", e))
            })?;
            Ok(true)
        }
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("AlreadyExists") || err_str.contains("already exists") {
                Ok(false)
            } else {
                Err(SpannerDbErr::Connection(format!("Failed to create database: {}", e)).into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_path_parse() {
        let path = DatabasePath::parse("projects/my-project/instances/my-instance/databases/my-db")
            .expect("Should parse valid path");

        assert_eq!(path.project, "my-project");
        assert_eq!(path.instance, "my-instance");
        assert_eq!(path.database, "my-db");
        assert_eq!(path.project_path(), "projects/my-project");
        assert_eq!(
            path.instance_path(),
            "projects/my-project/instances/my-instance"
        );
        assert_eq!(
            path.full_path(),
            "projects/my-project/instances/my-instance/databases/my-db"
        );
    }

    #[test]
    fn test_database_path_parse_invalid() {
        assert!(DatabasePath::parse("invalid/path").is_err());
        assert!(DatabasePath::parse("projects/p/instances/i").is_err());
        assert!(DatabasePath::parse("").is_err());
    }

    #[test]
    fn test_create_options_default() {
        let options = CreateOptions::default();
        assert!(!options.create_instance_if_not_exists);
        assert!(options.create_database_if_not_exists);
        assert_eq!(options.database_dialect, DatabaseDialect::GoogleStandardSql);
    }
}
