use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::{Pool, Postgres};
use std::process::Command;
use tokio::fs;

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct Video {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub video_url: String,
    pub hashtags: Vec<String>,
}