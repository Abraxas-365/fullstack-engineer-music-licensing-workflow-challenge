use std::sync::Arc;
use std::time::Duration;

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};
use sqlx::postgres::PgPoolOptions;

use backend::iam::auth::adapters::{
    JwtTokenService, PostgresSessionRepository, PostgresTokenRepository,
};
use backend::iam::auth::{AuthConfig, AuthService, JWTConfig, TokenService};
use backend::iam::role::RoleService;
use backend::iam::role::adapters::PostgresRoleRepository;
use backend::iam::user::UserService;
use backend::iam::user::adapters::{BcryptPasswordService, PostgresUserRepository};

use backend::label::LabelService;
use backend::label::adapters::PostgresLabelRepository;
use backend::license::LicenseService;
use backend::license::adapters::PostgresLicenseRepository;
use backend::movie::MovieService;
use backend::movie::adapters::PostgresMovieRepository;
use backend::scene::SceneService;
use backend::scene::adapters::PostgresSceneRepository;
use backend::song::SongService;
use backend::song::adapters::PostgresSongRepository;
use backend::track::TrackService;
use backend::track::adapters::PostgresTrackRepository;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // ── Database ────────────────────────────────────────────────────────────
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:5432/music_licensing".into());

    let pool = PgPoolOptions::new()
        .max_connections(
            std::env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        )
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // ── Migrations ──────────────────────────────────────────────────────────
    run_migrations(&pool).await;

    // ── Auth config ─────────────────────────────────────────────────────────
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        log::warn!("JWT_SECRET not set — using insecure default (dev only)");
        "dev-secret-key-change-in-production-must-be-32-chars".into()
    });

    let auth_config = AuthConfig {
        jwt: JWTConfig {
            secret_key: jwt_secret,
            access_token_ttl: Duration::from_secs(
                std::env::var("ACCESS_TOKEN_TTL_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(900),
            ),
            refresh_token_ttl: Duration::from_secs(
                std::env::var("REFRESH_TOKEN_TTL_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(604800),
            ),
            issuer: "music-licensing".into(),
        },
        ..Default::default()
    };

    // ── Repositories ────────────────────────────────────────────────────────
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let role_repo = Arc::new(PostgresRoleRepository::new(pool.clone()));
    let password_svc = Arc::new(BcryptPasswordService::new());
    let token_svc = Arc::new(JwtTokenService::new(&auth_config.jwt));
    let token_repo = Arc::new(PostgresTokenRepository::new(pool.clone()));
    let session_repo = Arc::new(PostgresSessionRepository::new(pool.clone()));

    let label_repo = Arc::new(PostgresLabelRepository::new(pool.clone()));
    let movie_repo = Arc::new(PostgresMovieRepository::new(pool.clone()));
    let scene_repo = Arc::new(PostgresSceneRepository::new(pool.clone()));
    let song_repo = Arc::new(PostgresSongRepository::new(pool.clone()));
    let track_repo = Arc::new(PostgresTrackRepository::new(pool.clone()));
    let license_repo = Arc::new(PostgresLicenseRepository::new(pool.clone()));

    // ── Services ────────────────────────────────────────────────────────────
    let auth_svc = web::Data::new(AuthService::new(
        user_repo.clone(),
        password_svc.clone(),
        token_svc.clone(),
        token_repo,
        session_repo,
        role_repo.clone(),
        auth_config,
    ));
    let token_svc_data = web::Data::from(token_svc as Arc<dyn TokenService>);
    let user_svc = web::Data::new(UserService::new(user_repo.clone(), password_svc.clone()));
    let role_svc = web::Data::new(RoleService::new(role_repo.clone(), user_repo.clone()));

    let label_svc = web::Data::new(LabelService::new(label_repo.clone(), user_repo.clone()));
    let movie_svc = web::Data::new(MovieService::new(movie_repo.clone(), user_repo.clone()));
    let scene_svc = web::Data::new(SceneService::new(scene_repo.clone(), movie_repo.clone()));
    let song_svc = web::Data::new(SongService::new(
        song_repo.clone(),
        user_repo.clone(),
        label_repo.clone(),
    ));
    let track_svc = web::Data::new(TrackService::new(
        track_repo.clone(),
        scene_repo.clone(),
        song_repo.clone(),
        movie_repo.clone(),
    ));
    let license_svc = web::Data::new(LicenseService::new(
        license_repo,
        track_repo,
        scene_repo,
        movie_repo,
        song_repo,
        label_repo,
    ));

    // ── HTTP server ─────────────────────────────────────────────────────────
    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let workers: usize = std::env::var("WORKERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0); // 0 = actix default (num CPUs)

    log::info!("Starting server on {}", bind);

    let mut server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req_head| {
                // In production, restrict to your frontend domain(s)
                let allowed = std::env::var("CORS_ORIGIN").unwrap_or_default();
                if allowed.is_empty() || allowed == "*" {
                    return true;
                }
                origin.as_bytes() == allowed.as_bytes()
            })
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec!["Authorization", "Content-Type"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            // Services
            .app_data(auth_svc.clone())
            .app_data(token_svc_data.clone())
            .app_data(user_svc.clone())
            .app_data(role_svc.clone())
            .app_data(label_svc.clone())
            .app_data(movie_svc.clone())
            .app_data(scene_svc.clone())
            .app_data(song_svc.clone())
            .app_data(track_svc.clone())
            .app_data(license_svc.clone())
            // Routes
            .service(
                web::scope("/api")
                    .configure(backend::iam::auth::api::configure)
                    .configure(backend::label::api::configure)
                    .configure(backend::movie::api::configure)
                    .configure(backend::scene::api::configure)
                    .configure(backend::song::api::configure)
                    .configure(backend::track::api::configure)
                    .configure(backend::license::api::configure),
            )
    })
    .shutdown_timeout(30);

    if workers > 0 {
        server = server.workers(workers);
    }

    server.bind(&bind)?.run().await
}

async fn run_migrations(pool: &sqlx::PgPool) {
    let has_tables: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'users')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if !has_tables {
        log::info!("Running migration 001_genesis");
        sqlx::raw_sql(include_str!("../migrations/001_genesis.up.sql"))
            .execute(pool)
            .await
            .expect("Failed to run migration 001");
    }

    log::info!("Running migration 002_seed_platform_roles (idempotent)");
    sqlx::raw_sql(include_str!("../migrations/002_seed_platform_roles.up.sql"))
        .execute(pool)
        .await
        .expect("Failed to run migration 002");
}
