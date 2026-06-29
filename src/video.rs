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

async fn process_video(input_path: &str, output_path: &str) -> Result<(), std::io::Error> {
    // Запускаем установленный в системе ffmpeg
    let status = Command::new("ffmpeg")
        .args(&[
            "-i", input_path,                // Входной файл
            "-vcodec", "libx265",            // Кодек H.265 (HEVC)
            "-crf", "28",                    // Хорошее сжатие для веса
            "-vf", "scale=540:960:force_original_aspect_ratio=decrease,pad=540:960:(ow-iw)/2:(oh-ih)/2", // Строго 9:16 и 540p
            "-acodec", "aac",                // Кодек звука
            "-y",                            // Перезаписывать, если файл есть
            output_path
        ])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "FFmpeg failed"))
    }
}

pub async fn upload_video(
    State(pool): State<Pool<Postgres>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut title = String::new();
    let mut hashtags_raw = String::new();
    let mut video_data = Vec::new();
    
    // Перебираем поля формы (текст и сам файл)
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        if name == "title" {
            title = field.text().await.unwrap();
        } else if name == "hashtags" {
            hashtags_raw = field.text().await.unwrap(); 
        } else if name == "video" {
            video_data = field.bytes().await.unwrap().to_vec();
        }
    }

    // Делим хэштеги по запятой
    let hashtags: Vec<String> = hashtags_raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Временный ID автора (пока нет сессий, ставим 1)
    let user_id = 1; 
    let temp_url = "processing"; 
    
    // 1. Вставляем запись в БД, чтобы сгенерировать ID для папки
    let row: (i32,) = sqlx::query_as(
        "INSERT INTO videos (user_id, title, video_url, hashtags) VALUES ($1, $2, $3, $4) RETURNING id"
    )
    .bind(user_id)
    .bind(&title)
    .bind(temp_url)
    .bind(&hashtags)
    .fetch_one(&pool)
    .await
    .unwrap();

    let video_id = row.0;

    // 2. Создаем структуру папок: static/videos/{video_id}/
    let dir_path = format!("static/videos/{}", video_id);
    fs::create_dir_all(&dir_path).await.unwrap();

    let temp_input = format!("{}/temp_input.mp4", dir_path);
    let final_output = format!("{}/video.mp4", dir_path);

    // Пишем сырой буфер во временный файл
    fs::write(&temp_input, video_data).await.unwrap();

    // 3. Запускаем кодирование
    if process_video(&temp_input, &final_output).await.is_ok() {
        // Избавляемся от исходника, чтобы экономить место
        let _ = fs::remove_file(&temp_input).await;

        let final_url = format!("/static/videos/{}/video.mp4", video_id);
        
        // Обновляем ссылку на готовое видео
        sqlx::query("UPDATE videos SET video_url = $1 WHERE id = $2")
            .bind(final_url)
            .bind(video_id)
            .execute(&pool)
            .await
            .unwrap();

        (StatusCode::OK, "Видео успешно загружено и обработано! (っ◕‿◕)っ")
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Не удалось перекодировать видео")
    }
}