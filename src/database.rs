use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub async fn init_db(database_url: &str) -> Pool<Postgres> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .unwrap()
}

pub async fn register_user(
    pool: &Pool<Postgres>, 
    username: &str, 
    password_hash: &str,
    email: &str,
    phone: &str
) -> Result<i32, sqlx::Error> {
    let row: (i32,) = sqlx::query_as(
        "INSERT INTO users (username, password_hash, email, phone) VALUES ($1, $2, $3, $4) RETURNING id"
    )
    .bind(username)
    .bind(password_hash)
    .bind(email)
    .bind(phone)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}