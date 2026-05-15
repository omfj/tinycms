use sqlx::postgres::PgPoolOptions;

pub type Pool = sqlx::PgPool;

pub async fn create_pool(url: &str) -> anyhow::Result<Pool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(url)
        .await?;

    sqlx::migrate!("../migrations").run(&pool).await?;

    Ok(pool)
}
