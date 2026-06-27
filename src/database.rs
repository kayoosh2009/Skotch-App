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

pub async fn get_user_for_login(
    pool: &Pool<Postgres>,
    login_identifier: &str
) -> Result<(i32, String), sqlx::Error> {
    let row: (i32, String) = sqlx::query_as(
        "SELECT id, password_hash FROM users WHERE username = $1 OR email = $1 OR phone = $1"
    )
    .bind(login_identifier)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

#[derive(serde::Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: String,
    pub phone: String,
}

#[derive(serde::Deserialize)]
pub struct LoginRequest {
    pub login_identifier: String,
    pub password: String,
}