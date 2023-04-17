use std::error::Error;
use sqlx::{Pool, sqlite::Sqlite, SqlitePool};
use teloxide::prelude::*;
use teloxide::types::UpdateKind;

#[derive(Debug, sqlx::FromRow)]
struct Chat {
    id: i32,
    name: String,
}

#[derive(Debug, sqlx::FromRow)]
struct Message {
    id: i32,
    text: String,
    chat_id: i32,
    user_id: i32,
}

pub struct Db {
    pool: Pool<Sqlite>,
}

impl Db {
    pub async fn new(url: String) -> Result<Self, Box<dyn Error>> {
        let url = format!("sqlite://{}", url);
        //std::env::set_var("DATABASE_URL", &url);
        let pool = SqlitePool::connect(&url).await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), Box<dyn Error>> {
        sqlx::query("CREATE TABLE IF NOT EXISTS chats (id INTEGER PRIMARY KEY, name TEXT)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS messages \
                (id INTEGER PRIMARY KEY, chat_id INTEGER, from_id TEXT, content TEXT, raw TEXT)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn save_updates(&self, updates: &Vec<Update>) -> Result<(), Box<dyn Error>> {
        for update in updates {
            let (id, chat_id, from_id, content, raw) = parse_update(update);

                sqlx::query("INSERT INTO messages (id, chat_id, from_id, content, raw) VALUES (?, ?, ?, ?, ?)")
                    .bind(id)
                    .bind(chat_id)
                    .bind(from_id)
                    .bind(content)
                    .bind(raw)
                    .execute(&self.pool)
                    .await?;
        }

        Ok(())
    }
}

fn parse_update(update: &Update) -> (i32, i64, Option<String>, Option<String>, Option<String>) {
    let id = update.id;
    let chat_id;
    let from_id;
    let content;
    let raw = None;//serde_json::to_string(update).unwrap_or("".to_string());

    match &update.kind {
        UpdateKind::Message(message) => {
            chat_id = message.chat.id.0;
            from_id = message.from().map_or(None, |user| Some(user.id.0.to_string()));
            content = message.text().map_or(None, |text| Some(text.to_string()));
        }
        //TODO: handle other update kinds
        _ => {
            chat_id = 0;
            from_id = None;
            content = None;
        }
    }

    (id, chat_id, from_id, content, raw)
}
