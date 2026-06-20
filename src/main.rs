use axum::{routing::get, Router};
use std::net::SocketAddr;

mod database;

#[tokio::main]
async fn main() {
    let database_url = "postgres://username:password@localhost/scotch_db";
    let pool = database::init_db(database_url).await;

    let app = Router::new()
        .route("/", get(index))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> &'static str {
    "Scotch API"
}