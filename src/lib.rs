pub mod api_routes;

use actix::{Actor, Addr};
use actix_web::{
  dev::{ServerHandle, ServiceResponse},
  middleware::{self, Condition, ErrorHandlerResponse, ErrorHandlers},
  web::{scope, Data},
  App, HttpResponse, HttpServer,
};
use clap::{Parser, Subcommand};
use lemmy_api_utils::site_snapshot::CachedSiteConfigProvider;
use lemmy_api_utils::{
  context::FastJobContext, request::client_builder,
  utils::local_site_rate_limit_to_rate_limit_config,
};
use lemmy_db_schema::{source::secret::Secret, utils::build_db_pool};
use lemmy_routes::{
  feeds,
  middleware::{
    idempotency::{IdempotencyMiddleware, IdempotencySet},
    session::SessionMiddleware,
  },
  nodeinfo,
  utils::{
    cors_config,
    prometheus_metrics::{new_prometheus_metrics, serve_prometheus},
    setup_local_site::setup_local_site,
  },
};
use lemmy_utils::redis::RedisClient;
use lemmy_utils::{
  error::FastJobResult,
  rate_limit::RateLimit,
  response::jsonify_plain_text_errors,
  settings::{structs::Settings, SETTINGS},
  VERSION,
};
use std::time::Duration;

use lemmy_routes::utils::scheduled_tasks::setup;
use mimalloc::MiMalloc;
use reqwest_middleware::ClientBuilder;
use reqwest_tracing::TracingMiddleware;
use serde_json::json;
use tokio::signal::unix::SignalKind;
use tracing_actix_web::{DefaultRootSpanBuilder, TracingLogger};
use lemmy_ws::broker::manager::PhoenixManager;
use lemmy_ws::presence::PresenceManager;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Timeout for HTTP requests while sending activities. A longer timeout provides better
/// compatibility with other ActivityPub software that might allocate more time for synchronous
/// processing of incoming activities. This timeout should be slightly longer than the time we
/// expect a remote server to wait before aborting processing on its own to account for delays from
/// establishing the HTTP connection and sending the request itself.
#[derive(Parser, Debug)]
#[command(
  version,
  about = "A link aggregator for the fediverse",
  long_about = "A link aggregator for the fediverse.\n\nThis is the FastJob backend API server. This will connect to a PostgreSQL database, run any pending migrations and start accepting API requests."
)]
// TODO: Instead of defining individual env vars, only specify prefix once supported by clap.
//       https://github.com/clap-rs/clap/issues/3221
pub struct CmdArgs {
  /// Don't run scheduled tasks.
  ///
  /// If you are running multiple Lemmy server processes, you probably want to disable scheduled
  /// tasks on all but one of the processes, to avoid running the tasks more often than intended.
  #[arg(long, default_value_t = false, env = "LEMMY_DISABLE_SCHEDULED_TASKS")]
  disable_scheduled_tasks: bool,
  /// Disables the HTTP server.
  ///
  /// This can be used to run a Lemmy server process that only performs scheduled tasks or activity
  /// sending.
  #[arg(long, default_value_t = false, env = "LEMMY_DISABLE_HTTP_SERVER")]
  disable_http_server: bool,
  /// Disable sending outgoing ActivityPub messages.
  ///
  /// Only pass this for horizontally scaled setups.
  /// See https://join-lemmy.org/docs/administration/horizontal_scaling.html for details.
  #[arg(long, default_value_t = false, env = "LEMMY_DISABLE_ACTIVITY_SENDING")]
  disable_activity_sending: bool,
  #[command(subcommand)]
  subcommand: Option<CmdSubcommand>,
}

#[derive(Subcommand, Debug)]
enum CmdSubcommand {
  /// Do something with migrations, then exit.
  Migration {
    #[command(subcommand)]
    subcommand: MigrationSubcommand,
    /// Stop after there's no remaining migrations.
    #[arg(long, default_value_t = false)]
    all: bool,
    /// Stop after the given number of migrations.
    #[arg(long, default_value_t = 1)]
    number: u64,
  },
}

#[derive(Subcommand, Debug)]
enum MigrationSubcommand {
  /// Run up.sql for pending migrations, oldest to newest.
  Run,
  /// Run down.sql for non-pending migrations, newest to oldest.
  Revert,
}

/// Placing the main function in lib.rs allows other crates to import it and embed Lemmy
pub async fn start_fastjob_server(args: CmdArgs) -> FastJobResult<()> {
  if let Some(CmdSubcommand::Migration {
    subcommand,
    all,
    number,
  }) = args.subcommand
  {
    let mut options = match subcommand {
      MigrationSubcommand::Run => lemmy_db_schema_setup::Options::default().run(),
      MigrationSubcommand::Revert => lemmy_db_schema_setup::Options::default().revert(),
    }
    .print_output();

    if !all {
      options = options.limit(number);
    }

    lemmy_db_schema_setup::run(options, &SETTINGS.get_database_url())?;

    return Ok(());
  }

  // Print version number to log
  println!("Starting FastJob v{VERSION}");

  // return error 503 while running db migrations and startup tasks
  let mut startup_server_handle = None;
  if !args.disable_http_server {
    startup_server_handle = Some(create_startup_server()?);
  }

  // Set up the connection pool
  let pool = build_db_pool()?;

  // Initialize the secrets
  let secret = Secret::init(&mut (&pool).into()).await?;

  // Make sure the local site is set up.
  let site_snapshot = setup_local_site(&mut (&pool).into(), &SETTINGS).await?;

  let site_config =
    CachedSiteConfigProvider::new(pool.clone(), site_snapshot.clone(), SETTINGS.clone());
  // site_config.clone().start_background_refresh(60);

  // Set up the rate limiter
  let rate_limit_config =
    local_site_rate_limit_to_rate_limit_config(&site_snapshot.site_view.local_site_rate_limit);
  let rate_limit_cell = RateLimit::new(rate_limit_config);

  println!(
    "Starting HTTP server at {}:{}",
    SETTINGS.bind, SETTINGS.port
  );

  let client = ClientBuilder::new(client_builder(&SETTINGS).build()?)
    .with(TracingMiddleware::default())
    .build();
  let pictrs_client = ClientBuilder::new(client_builder(&SETTINGS).no_proxy().build()?)
    .with(TracingMiddleware::default())
    .build();
  let conn_str = SETTINGS.get_redis_connection()?;
  let redis_client = RedisClient::new(&conn_str).await?;
  let scb_client = ClientBuilder::new(client_builder(&SETTINGS).no_proxy().build()?)
    .with(TracingMiddleware::default())
    .build();
  // Presence manager: timeout & sweep configuration
  let heartbeat_ttl = Duration::from_secs(45);
  // Start a lightweight system broker for broadcasting presence events
  let presence_manager =
    PresenceManager::new(heartbeat_ttl, Option::from(redis_client.clone())).start();
  let context = FastJobContext::create(
    pool.clone(),
    client.clone(),
    pictrs_client,
    secret.clone(),
    rate_limit_cell,
    redis_client.clone(),
    Box::new(site_config),
    scb_client,
  );

  // Phoenix manager needs presence address for online_users sync
  let phoenix_manager = PhoenixManager::new(
    SETTINGS.get_phoenix_url(),
    pool.clone(),
    presence_manager.clone(),
    redis_client.clone(),
  )
  .await
  .start();

  if let Some(prometheus) = SETTINGS.prometheus.clone() {
    serve_prometheus(prometheus, context.clone())?;
  }

  let server = if !args.disable_http_server {
    if let Some(startup_server_handle) = startup_server_handle {
      startup_server_handle.stop(true).await;
    }
    Some(create_http_server(
      context.clone(),
      phoenix_manager,
      SETTINGS.clone(),
    )?)
  } else {
    None
  };

  let mut interrupt = tokio::signal::unix::signal(SignalKind::interrupt())?;
  let mut terminate = tokio::signal::unix::signal(SignalKind::terminate())?;

  if let Err(err) = setup(Data::new(context.clone())).await {
    tracing::error!("Setup failed in HTTP init: {err:?}");
  }

  tokio::select! {
    _ = tokio::signal::ctrl_c() => {
      tracing::warn!("Received ctrl-c, shutting down gracefully...");
    }
    _ = interrupt.recv() => {
      tracing::warn!("Received interrupt, shutting down gracefully...");
    }
    _ = terminate.recv() => {
      tracing::warn!("Received terminate, shutting down gracefully...");
    }
  }
  if let Some(server) = server {
    server.stop(true).await;
  }

  Ok(())
}

/// Creates temporary HTTP server which returns status 503 for all requests.
fn create_startup_server() -> FastJobResult<ServerHandle> {
  let startup_server = HttpServer::new(move || {
    App::new().wrap(ErrorHandlers::new().default_handler(move |req| {
      let (req, _) = req.into_parts();
      let response =
        HttpResponse::ServiceUnavailable().json(json!({"error": "FastJob is currently starting"}));
      let service_response = ServiceResponse::new(req, response);
      Ok(ErrorHandlerResponse::Response(
        service_response.map_into_right_body(),
      ))
    }))
  })
  .bind((SETTINGS.bind, SETTINGS.port))?
  .run();
  let startup_server_handle = startup_server.handle();
  tokio::task::spawn(startup_server);
  Ok(startup_server_handle)
}

fn create_http_server(
  context: FastJobContext,
  phoenix_manager: Addr<PhoenixManager>,
  settings: Settings,
) -> FastJobResult<ServerHandle> {
  // These must come before HttpServer creation so they can collect data across threads.
  let prom_api_metrics = new_prometheus_metrics()?;
  let idempotency_set = IdempotencySet::default();
  // Create Http server
  let bind = (settings.bind, settings.port);
  let server = HttpServer::new(move || {
    let rate_limit = context.rate_limit_cell().clone();
    let cors_config = cors_config(&settings);

    // Create a more efficient middleware stack with optimized ordering
    // - Put frequently used middleware first (compression, CORS)
    // - Group related middleware together
    // - Use conditional middleware only when needed
    let app = App::new()
      // Compression should be first to reduce data transfer size
      .wrap(middleware::Compress::default())
      // CORS headers are checked early in request processing
      .wrap(cors_config)
      // Session middleware should be early as it's used by most routes
      .wrap(SessionMiddleware::new(context.clone()))
      // Idempotency middleware prevents duplicate operations
      .wrap(IdempotencyMiddleware::new(idempotency_set.clone()))
      // Error handlers should be after business logic middleware
      .wrap(ErrorHandlers::new().default_handler(jsonify_plain_text_errors))
      // Logging and tracing should be last to capture the full request lifecycle
      .wrap(TracingLogger::<DefaultRootSpanBuilder>::new())
      .wrap(middleware::Logger::new(
        // This is the default log format save for the usage of %{r}a over %a to guarantee to
        // record the client's (forwarded) IP and not the last peer address, since the latter is
        // frequently just a reverse proxy
        "%{r}a '%r' %s %b '%{Referer}i' '%{User-Agent}i' %T",
      ))
      // Conditional middleware for metrics
      .wrap(Condition::new(
        SETTINGS.prometheus.is_some(),
        prom_api_metrics.clone(),
      ))
      // Application data - these don't affect middleware order
      .app_data(Data::new(context.clone()))
      .app_data(Data::new(phoenix_manager.clone()));

    app
      .configure(|cfg| api_routes::config(cfg, &rate_limit))
      .configure(feeds::config)
      .configure(nodeinfo::config)
      .service(scope("/sitemap.xml").wrap(rate_limit.message()))
  })
  // Use number of available CPU cores for optimal performance
  .workers(
    std::thread::available_parallelism()
      .map(|p| p.get())
      .unwrap_or(2),
  )
  .disable_signals()
  // Limit the number of concurrent connections to prevent too many open files
  // Increased from 1000 to 2000 for better handling of high traffic
  .max_connections(2000)
  // Reduced keep-alive timeout to close idle connections faster and free up resources
  .keep_alive(std::time::Duration::from_secs(15))
  .bind(bind)?
  .run();
  let handle = server.handle();
  tokio::task::spawn(server);
  Ok(handle)
}
