use std::error::Error;
use std::str::FromStr;
use sqlx::{Execute, Pool, Row, sqlite::Sqlite, SqlitePool};
use teloxide::prelude::*;
use teloxide::types::UpdateKind;

#[derive(Debug, sqlx::FromRow)]
struct Users {
    id: i32,
    name: String,
}

struct Permissions {
    id: i32,
    user_id: i32,
    chat_id: i32,
    is_bot_admin: bool,
    custom_tag: Option<String>,
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
        sqlx::query("CREATE TABLE IF NOT EXISTS conf (key TEXT PRIMARY KEY, value TEXT)").execute(&self.pool).await?;
        sqlx::query("INSERT OR IGNORE INTO conf (key, value) VALUES ('OFFSET', NULL)").execute(&self.pool).await?;

        sqlx::query("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)").execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS messages \
                (id INTEGER PRIMARY KEY, chat_id INTEGER, from_id TEXT, content TEXT, raw TEXT)").execute(&self.pool).await?;

        Ok(())
    }

    pub async fn read_conf_value<T>(&self, key: &str) -> Result<Option<T>, Box<dyn Error>>
        where T: FromStr,
              <T as FromStr>::Err: Error {
        let Some(string_value) = self.read_conf_value_raw(key).await? else { return Ok(None); };

        match string_value.parse::<T>() {
            Ok(value) => Ok(Some(value)),
            Err(_) => Err(format!("Error parsing value by key '{}', value: '{}'", key, string_value).into())
        }

        //Ok(Some(value))
    }

    async fn read_conf_value_raw(&self, key: &str) -> Result<Option<String>, Box<dyn Error>> {
        let conf = sqlx::query("SELECT value FROM conf WHERE key = ?").bind(key).fetch_optional(&self.pool).await?;

        let Some(row) = conf else { return Ok(None); };

        let value = row.try_get::<Option<String>, usize>(0)?;
        let Some(value) = value else { return Ok(None); };

        Ok(Some(value.clone()))
    }

    pub async fn write_conf_value<'q, T>(&'q self, key: &'q str, value: Option<T>) -> Result<(), sqlx::Error>
        where
            T: 'q + Send + sqlx::Encode<'q, Sqlite> + sqlx::Type<Sqlite> {
        sqlx::query("UPDATE conf SET value = ? WHERE key = ?")
            .bind(value)
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn save_updates(&self, updates: &Vec<Update>) -> Result<(), Box<dyn Error>> {
        for update in updates {
            let (id, chat_id, from_id, content, raw) = parse_update(update);

            sqlx::query("INSERT INTO messages (id, chat_id, from_id, content, raw) VALUES (?, ?, ?, ?, ?)").bind(id).bind(chat_id).bind(from_id).bind(content).bind(raw).execute(&self.pool).await?;
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
