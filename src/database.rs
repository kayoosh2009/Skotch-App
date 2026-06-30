use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use rand::rngs::OsRng;
use rand::RngCore;

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
    password: &str,
    email: &str,
    phone: &str
) -> Result<i32, sqlx::Error> {
    let password_hash = hash_password(password)
        .map_err(|e| sqlx::Error::Protocol(format!("Ошибка хеширования пароля: {}", e)))?;

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

fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub async fn check_login(
    pool: &Pool<Postgres>,
    login_identifier: &str,
    password: &str,
) -> Result<i32, sqlx::Error> {
    let row: (i32, String) = sqlx::query_as(
        "SELECT id, password_hash FROM users WHERE username = $1 OR email = $1 OR phone = $1"
    )
    .bind(login_identifier)
    .fetch_one(pool)
    .await?;

    let (user_id, password_hash) = row;

    if verify_password(password, &password_hash) {
        Ok(user_id)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
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

#[derive(serde::Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: i32,
}

pub async fn create_session(pool: &Pool<Postgres>, user_id: i32) -> Result<String, sqlx::Error> {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let token = hex::encode(bytes);

    sqlx::query("INSERT INTO sessions (token, user_id) VALUES ($1, $2)")
        .bind(&token)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(token)
}