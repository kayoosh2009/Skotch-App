use axum::{
    routing::{get, post}, 
    Router, 
    Json, 
    extract::State, 
    http::StatusCode, 
    response::IntoResponse
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use sqlx::{Pool, Postgres};

mod database;

mod database;

#[derive(serde::Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String, // Фронтенд шлет чистый пароль, хэшировать будем в хендлере
    pub email: String,
    pub phone: String,
}

#[tokio::main]
async fn main() {
    // Раскомментируем подключение к БД (укажи свои реальные данные)
    let database_url = "postgres://username:password@localhost/scotch_db";
    let pool = database::init_db(database_url).await;

    let app = Router::new()
        .route("/", get(index))
        // Добавляем роут POST-запроса для регистрации
        .route("/auth/register", post(register))
        // Отдаем статические файлы
        .nest_service("/static", ServeDir::new("static"))
        // Передаем пул базы данных как состояние для всего роутера
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Сервер запущен на http://{}", addr);
    println!("Страница регистрации доступна по адресу: http://{}/static/user/register.html", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn register(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<database::RegisterRequest>,
) -> impl IntoResponse {
    // В реальном проекте здесь нужно добавить хэширование пароля (например, через bcrypt или argon2)
    let password_hash = payload.password; 

    match database::register_user(
        &pool, 
        &payload.username, 
        &password_hash, 
        &payload.email, 
        &payload.phone
    ).await {
        Ok(_) => (StatusCode::OK, "Регистрация успешна".into_response()),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR, 
            format!("Ошибка базы данных: {}", e).into_response()
        ),
    }
}

async fn index() -> &'static str {
    "Scotch API работает. Перейдите на /static/user/register.html"
}