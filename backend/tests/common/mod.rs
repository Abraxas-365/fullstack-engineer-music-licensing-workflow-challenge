use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers_modules::postgres::Postgres;

pub struct TestDb {
    pub pool: PgPool,
    _container: ContainerAsync<Postgres>,
}

impl TestDb {
    pub async fn new() -> Self {
        let container = Postgres::default()
            .start()
            .await
            .expect("Failed to start postgres container");

        let port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("Failed to get postgres port");

        let connection_string =
            format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

        let pool = PgPool::connect(&connection_string)
            .await
            .expect("Failed to connect to postgres");

        let migration = include_str!("../../migrations/001_genesis.up.sql");
        sqlx::raw_sql(migration)
            .execute(&pool)
            .await
            .expect("Failed to run migration");

        Self {
            pool,
            _container: container,
        }
    }
}
