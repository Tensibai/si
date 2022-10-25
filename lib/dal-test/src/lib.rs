//! This module provides helpers and resources for constructing and running integration tests.

use std::{
    borrow::Cow,
    env,
    path::{Path, PathBuf},
    sync::{Arc, Once},
};

#[cfg(debug_assertions)]
use dal::check_runtime_dependencies;
use dal::{
    job::processor::{sync_processor::SyncProcessor, JobQueueProcessor},
    DalContext, JwtSecretKey, ServicesContext,
};
use lazy_static::lazy_static;
use si_data::{NatsClient, NatsConfig, PgPool, PgPoolConfig, ResultExt};
use telemetry::prelude::*;
use tokio::sync::Mutex;
use uuid::Uuid;
use veritech_client::EncryptionKey;
use veritech_server::{Instance, StandardConfig};

pub use color_eyre::{
    self,
    eyre::{eyre, Result, WrapErr},
};
pub use si_test_macros::dal_test as test;
pub use telemetry;
pub use tracing_subscriber;

pub mod helpers;
pub mod test_harness;

#[cfg(debug_assertions)]
pub const CANONICALIZE_CYCLONE_BIN_PATH_ERROR_MESSAGE: &str =
    "failed to canonicalize cyclone bin path (you likely need to build cyclone: cargo build --bin cyclone)";
#[cfg(not(debug_assertions))]
pub const CANONICALIZE_CYCLONE_BIN_PATH_ERROR_MESSAGE: &str =
    "failed to canonicalize cyclone bin path";

const DEFAULT_PG_DBNAME: &str = "si_test";

const ENV_VAR_NATS_URL: &str = "SI_TEST_NATS_URL";
const ENV_VAR_PG_HOSTNAME: &str = "SI_TEST_PG_HOSTNAME";
const ENV_VAR_PG_DBNAME: &str = "SI_TEST_PG_DBNAME";

const JWT_PUBLIC_FILENAME: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", "config/public.pem");
const JWT_PRIVATE_FILENAME: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", "config/private.pem");

pub static COLOR_EYRE_INIT: Once = Once::new();

lazy_static! {
    static ref CONFIG: Config = Config::default();
    static ref TEST_CONTEXT_BUILDER: Mutex<ContextBuilderState> = Mutex::new(Default::default());
}

/// A [`DalContext`] for a workspace in a billing account which is not in a change set nor an edit
/// session.
///
/// To use a borrowed `DalContext` version, use [`DalContextHeadRef`].
/// To use mutably borrowed `DalContext` version, use [`DalContextHeadMutRef`].
pub struct DalContextHead(pub DalContext);

/// A reference to a [`DalContext`] for a workspace in a billing account which is not in a change
/// set nor an edit session.
///
/// To use an owned `DalContext` version, use [`DalContextHead`].
/// To use mutably borrowed `DalContext` version, use [`DalContextHeadMutRef`].
pub struct DalContextHeadRef<'a>(pub &'a DalContext);

/// A mutable reference to a [`DalContext`] for a workspace in a billing account which is not in a
/// change set nor an edit session.
///
/// To use an owned `DalContext` version, use [`DalContextHead`].
/// To use a borrowed `DalContext` version, use [`DalContextHeadRef`].
pub struct DalContextHeadMutRef<'a>(pub &'a mut DalContext);

/// A [`DalContext`] with universal read/write tenancies and a head visibility.
///
/// To use a borrowed `DalContext` version, use [`DalContextUniversalHeadRef`].
/// To use mutably borrowed `DalContext` version, use [`DalContextUniversalHeadMutRef`].
pub struct DalContextUniversalHead(pub DalContext);

/// A reference to a [`DalContext`] with universal read/write tenancies and a head visibility.
///
/// To use an owned `DalContext` version, use [`DalContextUniversalHead`].
/// To use mutably borrowed `DalContext` version, use [`DalContextUniversalHeadMutRef`].
pub struct DalContextUniversalHeadRef<'a>(pub &'a DalContext);

/// A mutable reference to a [`DalContext`] with universal read/write tenancies and a head
/// visibility.
///
/// To use an owned `DalContext` version, use [`DalContextUniversalHead`].
/// To use a borrowed `DalContext` version, use [`DalContextUniversalHeadRef`].
pub struct DalContextUniversalHeadMutRef<'a>(pub &'a mut DalContext);

#[derive(Clone, Debug)]
pub struct Config {
    pg_pool: PgPoolConfig,
    nats: NatsConfig,
    encryption_key_path: PathBuf,
}

impl Config {
    fn default_encryption_key_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../cyclone-server/src/dev.encryption.key")
    }

    fn default_nats() -> NatsConfig {
        let mut nats = NatsConfig::default();
        if let Ok(value) = env::var(ENV_VAR_NATS_URL) {
            nats.url = value;
        }
        nats
    }

    fn default_pg_pool() -> PgPoolConfig {
        let mut pg_pool = PgPoolConfig::default();
        if let Ok(value) = env::var(ENV_VAR_PG_HOSTNAME) {
            pg_pool.hostname = value;
        }
        pg_pool.dbname =
            env::var(ENV_VAR_PG_DBNAME).unwrap_or_else(|_| DEFAULT_PG_DBNAME.to_string());
        pg_pool.pool_max_size *= 32;
        pg_pool
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pg_pool: Self::default_pg_pool(),
            nats: Self::default_nats(),
            encryption_key_path: Self::default_encryption_key_path(),
        }
    }
}

#[allow(clippy::large_enum_variant)]
enum ContextBuilderState {
    Uninitialized,
    Created(TestContextBuilder),
    Errored(Cow<'static, str>),
}

impl ContextBuilderState {
    fn created(builder: TestContextBuilder) -> Self {
        Self::Created(builder)
    }

    fn errored(message: impl Into<Cow<'static, str>>) -> Self {
        Self::Errored(message.into())
    }
}

impl Default for ContextBuilderState {
    fn default() -> Self {
        Self::Uninitialized
    }
}

/// A context used for preparing and running tests containing DAL objects.
#[derive(Clone, Debug)]
pub struct TestContext {
    /// The test context configuration used to build this instance.
    config: Config,
    /// A PostgreSQL connection pool.
    pg_pool: PgPool,
    /// A connected NATS client
    nats_conn: NatsClient,
    /// A [`JobQueueProcessor`] impl
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    /// A key for re-recrypting messages to the function execution system.
    encryption_key: Arc<EncryptionKey>,
    /// A key used to decrypt the JWT signing key from the database.
    jwt_secret_key: JwtSecretKey,
}

impl TestContext {
    /// Builds and returns a suitable [`TestContext`] from a global configuration which is ready to
    /// run tests.
    ///
    /// # Implementation Details
    ///
    /// This functions wraps over a mutex which ensures that only the first caller will run global
    /// database creation, migrations, and other preparations.
    pub async fn global() -> Result<Self> {
        let mut mutex_guard = TEST_CONTEXT_BUILDER.lock().await;

        match &*mutex_guard {
            ContextBuilderState::Uninitialized => {
                let test_context_builder = TestContextBuilder::create(CONFIG.clone())
                    .await
                    .si_inspect_err(|err| {
                        *mutex_guard = ContextBuilderState::errored(err.to_string());
                    })?;

                // The stack gets too deep here, so we'll spawn the work as a task with a new
                // thread stack just for the global setup
                let handle = tokio::spawn(global_setup(test_context_builder.clone()));

                // Join this task and wait on its completion
                match handle.await {
                    // Global setup completed successfully
                    Ok(Ok(())) => {
                        debug!("task global_setup was successful");
                        *mutex_guard = ContextBuilderState::created(test_context_builder.clone());
                        test_context_builder.build().await
                    }
                    // Global setup errored
                    Ok(Err(err)) => {
                        *mutex_guard = ContextBuilderState::errored(err.to_string());
                        Err(err)
                    }
                    // Tokio task panicked or was cancelled
                    Err(err) => {
                        if err.is_panic() {
                            error!(error = %err, "spawned task global_setup panicked!");
                        } else if err.is_cancelled() {
                            error!(error = %err, "spawned task global_setup was cancelled!");
                        }
                        *mutex_guard = ContextBuilderState::errored(err.to_string());
                        Err(err.into())
                    }
                }
            }
            ContextBuilderState::Created(builder) => builder.build().await,
            ContextBuilderState::Errored(message) => {
                error!(error = %message, "global setup failed, aborting test");
                Err(eyre!("global setup failed: {}", message))
            }
        }
    }

    /// Creates a new [`ServicesContext`] using the given NATS subject prefix.
    pub async fn create_services_context(
        &self,
        nats_subject_prefix: impl Into<String>,
    ) -> ServicesContext {
        let veritech = veritech_client::Client::with_subject_prefix(
            self.nats_conn.clone(),
            nats_subject_prefix,
        );

        ServicesContext::new(
            self.pg_pool.clone(),
            self.nats_conn.clone(),
            self.job_processor.clone(),
            veritech,
            self.encryption_key.clone(),
        )
    }

    /// Gets a reference to the NATS configuration.
    pub fn nats_config(&self) -> &NatsConfig {
        &self.config.nats
    }

    /// Gets a reference to the JWT secret key.
    pub fn jwt_secret_key(&self) -> &JwtSecretKey {
        &self.jwt_secret_key
    }
}

/// A builder for a [`TestContext`].
///
/// Each `TestContext` has an active connection pool to the database and messaging system, and
/// rather than share these single pools among all global set and all test tests, a new set of
/// dedicated pools can be created as needed. This builder holds all other state other than the
/// pool-acquiring steps.
#[derive(Clone, Debug)]
struct TestContextBuilder {
    /// The test context configuration used to build this instance.
    config: Config,
    /// A key for re-recrypting messages to the function execution system.
    encryption_key: Arc<EncryptionKey>,
    /// A key used to decrypt the JWT signing key from the database.
    jwt_secret_key: JwtSecretKey,
}

impl TestContextBuilder {
    /// Creates a new builder.
    async fn create(config: Config) -> Result<Self> {
        let encryption_key = Arc::new(
            EncryptionKey::load(&config.encryption_key_path)
                .await
                .wrap_err("failed to load EncryptionKey")?,
        );
        let jwt_secret_key = JwtSecretKey::default();

        Ok(Self {
            config,
            encryption_key,
            jwt_secret_key,
        })
    }

    /// Builds and returns a new [`TestContext`] with its own connection pooling.
    async fn build(&self) -> Result<TestContext> {
        let pg_pool = PgPool::new(&self.config.pg_pool)
            .await
            .wrap_err("failed to create PgPool")?;
        let nats_conn = NatsClient::new(&self.config.nats)
            .await
            .wrap_err("failed to create NatsClient")?;
        let job_processor =
            Box::new(SyncProcessor::new()) as Box<dyn JobQueueProcessor + Send + Sync>;

        Ok(TestContext {
            config: self.config.clone(),
            pg_pool,
            nats_conn,
            job_processor,
            encryption_key: self.encryption_key.clone(),
            jwt_secret_key: self.jwt_secret_key.clone(),
        })
    }
}

/// Generates a new pseudo-random NATS subject prefix.
pub fn nats_subject_prefix() -> String {
    Uuid::new_v4().as_simple().to_string()
}

/// Configures and builds a [`veritech_server::Server`] suitable for running alongside DAL object-related
/// tests.
pub async fn veritech_server_for_uds_cyclone(
    nats_config: NatsConfig,
    nats_subject_prefix: String,
) -> Result<veritech_server::Server> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cyclone_spec = veritech_server::CycloneSpec::LocalUds(
        veritech_server::LocalUdsInstance::spec()
            .try_cyclone_cmd_path(
                dir.join("../../target/debug/cyclone")
                    .canonicalize()
                    .wrap_err(CANONICALIZE_CYCLONE_BIN_PATH_ERROR_MESSAGE)?
                    .to_string_lossy()
                    .to_string(),
            )
            .wrap_err("failed to setup cyclone_cmd_path")?
            .cyclone_decryption_key_path(
                dir.join("../../lib/cyclone-server/src/dev.decryption.key")
                    .canonicalize()
                    .wrap_err("failed to canonicalize cyclone decryption key path")?
                    .to_string_lossy()
                    .to_string(),
            )
            .try_lang_server_cmd_path(
                dir.join("../../bin/lang-js/target/lang-js")
                    .canonicalize()
                    .wrap_err("failed to canonicalize lang-js path")?
                    .to_string_lossy()
                    .to_string(),
            )
            .wrap_err("failed to setup lang_js_cmd_path")?
            .all_endpoints()
            .build()
            .wrap_err("failed to build cyclone spec")?,
    );
    let config = veritech_server::Config::builder()
        .nats(nats_config)
        .subject_prefix(nats_subject_prefix)
        .cyclone_spec(cyclone_spec)
        .build()
        .wrap_err("failed to build spec")?;
    let server = veritech_server::Server::for_cyclone_uds(config)
        .await
        .wrap_err("failed to create Veritech server")?;

    Ok(server)
}

async fn global_setup(test_context_builer: TestContextBuilder) -> Result<()> {
    info!("running global test setup");
    let test_context = test_context_builer.build().await?;

    debug!("initializing crypto");
    sodiumoxide::init().map_err(|_| eyre!("failed to init sodiumoxide crypto"))?;

    #[cfg(debug_assertions)]
    {
        info!("checking for required runtime dependencies");
        check_runtime_dependencies()?;
    }

    let nats_subject_prefix = nats_subject_prefix();

    // Start up a Veritech server as a task exclusively to allow the migrations to run
    info!("starting Veritech server for initial migrations");
    let veritech_server = veritech_server_for_uds_cyclone(
        test_context.config.nats.clone(),
        nats_subject_prefix.clone(),
    )
    .await?;
    let veritech_server_handle = veritech_server.shutdown_handle();
    tokio::spawn(veritech_server.run());

    // Create a `ServicesContext`
    let services_ctx = test_context
        .create_services_context(nats_subject_prefix)
        .await;

    info!("testing database connection");
    services_ctx
        .pg_pool()
        .test_connection()
        .await
        .wrap_err("failed to connect to database, is it running and available?")?;

    // Ensure the database is totally clean, then run all migrations
    info!("dropping and re-creating the database schema");
    services_ctx
        .pg_pool()
        .drop_and_create_public_schema()
        .await
        .wrap_err("failed to drop and create the database")?;
    info!("running database migrations");
    dal::migrate(services_ctx.pg_pool())
        .await
        .wrap_err("failed to migrate database")?;

    // Setup the JWT-signing key in the database
    {
        info!("creating jwt key in database");
        let mut pg_conn = services_ctx
            .pg_pool()
            .get()
            .await
            .wrap_err("failed to get pg connection")?;
        let pg_txn = pg_conn
            .transaction()
            .await
            .wrap_err("failed to start pg transaction")?;
        dal::create_jwt_key_if_missing(
            &pg_txn,
            JWT_PUBLIC_FILENAME,
            JWT_PRIVATE_FILENAME,
            &test_context.jwt_secret_key.key,
        )
        .await
        .wrap_err("failed to create jwt key")?;
        pg_txn
            .commit()
            .await
            .wrap_err("failed to commit jwt key insertion txn")?;
    }

    info!("creating builtins");
    dal::migrate_builtins(
        services_ctx.pg_pool(),
        services_ctx.nats_conn(),
        services_ctx.job_processor(),
        services_ctx.veritech().clone(),
        services_ctx.encryption_key(),
    )
    .await
    .wrap_err("failed to run builtin migrations")?;

    // Shutdown the Veritech server (each test gets their own server instance with an exclusively
    // unique subject prefix)
    info!("shutting down initial migrations Veritech server");
    veritech_server_handle.shutdown().await;

    info!("global test setup complete");
    Ok(())
}