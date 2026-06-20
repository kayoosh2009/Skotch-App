use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

mod database;

#[tokio::main]
async fn main() {
    // Временно комментируем подключение к БД, чтобы сервер не падал при запуске без запущенного Postgres
    /*
    let database_url = "postgres://username:password@localhost/scotch_db";
    let pool = database::init_db(database_url).await;
    */

    let app = Router::new()
        .route("/", get(index))
        // Этот метод заставит Axum отдавать файлы из папки static по пути /static/*
        .nest_service("/static", ServeDir::new("static"));
        // .with_state(pool); // Тоже временно скрываем, пока нет пула

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Сервер запущен на http://{}", addr);
    println!("Страница регистрации доступна по адресу: http://{}/static/user/register.html", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> &'static str {
    "Scotch API работает. Перейдите на /static/user/register.html"
}