# Установка и запуск проекта Scotch на Fedora

Инструкция для первого запуска проекта: системные зависимости, PostgreSQL, таблицы в БД и сборка.

## 1. Системные зависимости

```bash
sudo dnf update -y
sudo dnf install -y gcc gcc-c++ make pkg-config openssl-devel
```

Rust (если ещё не установлен):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version
cargo --version
```

ffmpeg (нужен для обработки видео в `video.rs`; в стандартных репозиториях Fedora его нет — ставим из RPM Fusion):

```bash
sudo dnf install -y https://download1.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm
sudo dnf install -y https://download1.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-$(rpm -E %fedora).noarch.rpm
sudo dnf install -y ffmpeg
ffmpeg -version
```

## 2. PostgreSQL

```bash
sudo dnf install -y postgresql postgresql-server postgresql-contrib
sudo postgresql-setup --initdb
sudo systemctl enable --now postgresql
```

Создаём базу и пользователя (под локальным юзером, под которым будешь запускать `cargo run`):

```bash
sudo -u postgres psql -c "CREATE USER $USER WITH SUPERUSER PASSWORD 'postgres';"
sudo -u postgres psql -c "CREATE DATABASE scotch_db OWNER $USER;"
```

По умолчанию Fedora настраивает PostgreSQL на `peer`/`ident`-аутентификацию для локальных соединений, что может конфликтовать с паролем. Если будут проблемы с подключением, открой `/var/lib/pgsql/data/pg_hba.conf` и поменяй `peer`/`ident` на `md5` для строк `local` и `127.0.0.1`, затем перезапусти службу:

```bash
sudo systemctl restart postgresql
```

## 3. Таблицы в базе данных

Подключаемся:

```bash
psql -U $USER -d scotch_db -h localhost
```

В открывшемся `psql` выполняем:

```sql
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    phone TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS videos (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    title TEXT NOT NULL,
    video_url TEXT NOT NULL,
    hashtags TEXT[] NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS sessions (
    token TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

Выход: `\q`

## 4. Cargo.toml

Проверь, что в проекте есть все нужные зависимости:

```toml
[package]
name = "scotch"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = { version = "0.7", features = ["multipart"] }
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres"] }
tower-http = { version = "0.5", features = ["fs"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
argon2 = "0.5"
rand = "0.8"
hex = "0.4"
```

Если версии в реальном проекте отличаются — не трогай уже работающие строки, добавь только недостающие (`argon2`, `rand`, `hex`).

## 5. Строка подключения к БД

В `main.rs` строка подключения должна включать пользователя и пароль, которые ты создал на шаге 2:

```
postgres://USERNAME:postgres@localhost/scotch_db
```

(замени `USERNAME` на своё имя пользователя в системе).

## 6. Запуск проекта

```bash
cd ~/путь/к/проекту
mkdir -p static/user static/videos
cargo build
cargo run
```

Первая сборка скачает и скомпилирует все зависимости — на Fedora это может занять несколько минут из-за компиляции `sqlx` и `argon2`.

## Возможные проблемы

- **`error: linking with cc failed`** — не хватает `gcc` или `openssl-devel`, проверь шаг 1.
- **`password authentication failed for user`** — проверь шаг 2 про `pg_hba.conf` и `md5`.
- **`ffmpeg: command not found`** — RPM Fusion не подключился, проверь шаг 1 (ffmpeg).
- **Видео не загружается / папка не создаётся** — проверь, что выполнил `mkdir -p static/user static/videos` перед запуском, и что у пользователя есть права на запись в эту директорию.