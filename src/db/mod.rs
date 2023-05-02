use std::error::Error;
use std::str::FromStr;
use sqlx::{Pool, Row, sqlite::Sqlite, SqlitePool};
use teloxide::prelude::*;

mod messages;
mod permissions;

pub struct Db {
    pool: Pool<Sqlite>,
}

pub enum ConfKey {
    Offset,
    ChatId,
}

impl ConfKey {
    fn get_db_key(&self) -> &'static str {
        match self {
            ConfKey::Offset => "OFFSET",
            ConfKey::ChatId => "CHAT_ID",
        }
    }
}

impl Db {
    pub async fn new(url: String) -> Result<Self, Box<dyn Error>> {
        let url = format!("sqlite://{}", url);
        let pool = SqlitePool::connect(&url).await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), Box<dyn Error>> {
        sqlx::query("CREATE TABLE IF NOT EXISTS conf (key TEXT PRIMARY KEY, value TEXT)").execute(&self.pool).await?;
        sqlx::query(
            "INSERT OR IGNORE INTO conf (key, value) VALUES \
            ('OFFSET', NULL), \
            ('CHAT_ID', NULL) \
            ").execute(&self.pool).await?;

        //sqlx::query("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)").execute(&self.pool).await?;

        messages::Message::create_table(&self.pool).await?;

        Ok(())
    }

    pub async fn read_conf_value<T>(&self, key: ConfKey) -> Result<Option<T>, Box<dyn Error>>
        where T: FromStr,
              <T as FromStr>::Err: Error {
        let key = key.get_db_key();
        let Some(string_value) = self.read_conf_value_raw(key).await? else { return Ok(None); };

        match string_value.parse::<T>() {
            Ok(value) => Ok(Some(value)),
            Err(_) => Err(format!("Error parsing value by key '{}', value: '{}'", key, string_value).into())
        }
    }

    async fn read_conf_value_raw(&self, key: &str) -> Result<Option<String>, Box<dyn Error>> {
        let conf = sqlx::query("SELECT value FROM conf WHERE key = ?").bind(key).fetch_optional(&self.pool).await?;

        let Some(row) = conf else { return Ok(None); };

        let value = row.try_get::<Option<String>, usize>(0)?;
        let Some(value) = value else { return Ok(None); };

        Ok(Some(value.clone()))
    }

    pub async fn write_conf_value<'q, T>(&self, key: ConfKey, value: Option<T>) -> Result<(), sqlx::Error>
        where
            T: 'q + Send + sqlx::Encode<'q, Sqlite> + sqlx::Type<Sqlite> {
        let key = key.get_db_key();
        sqlx::query("UPDATE conf SET value = ? WHERE key = ?")
            .bind(value)
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub(crate) async fn set_bot_admin(&self, _user_id: u64) -> Result<Option<String>, Box<dyn Error>> {
        todo!()
    }

    pub async fn save_updates(&self, updates: &Vec<Update>) -> Result<(), Box<dyn Error>> {
        for update in updates {
            let msg = messages::Message::from(update);
            msg.insert(&self.pool).await?;
        }

        Ok(())
    }
}
