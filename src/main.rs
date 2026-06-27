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

#[tokio::main]
async fn main() {
    // Используем явную авторизацию по логину и паролю
    let database_url = "postgres://localhost/scotch_db";
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
    let password_hash = payload.password; 

    match database::register_user(
        &pool, 
        &payload.username, 
        &password_hash, 
        &payload.email, 
        &payload.phone
    ).await {
        Ok(_) => (StatusCode::OK, "Регистрация успешна".into_response()),
        Err(e) => {
            // Проверяем, является ли ошибка ошибкой базы данных
            if let Some(db_err) = e.as_database_error() {
                // Код 23505 в PostgreSQL означает нарушение уникальности (Unique Violation)
                if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                    return (
                        StatusCode::BAD_REQUEST, 
                        "Этот логин, почта или телефон уже зарегистрированы".into_response()
                    );
                }
            }
            
            // Для всех остальных непредвиденных ошибок возвращаем 500
            (
                StatusCode::INTERNAL_SERVER_ERROR, 
                "Произошла внутренняя ошибка сервера".into_response()
            )
        }
    }
}

async fn index() -> &'static str {
    "Scotch API работает. Перейдите на /static/user/register.html"
}