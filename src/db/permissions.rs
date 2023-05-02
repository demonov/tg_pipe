use sqlx::{FromRow, SqlitePool};
use teloxide::prelude::*;

#[derive(Debug, FromRow)]
pub struct Permissions {
    user_id: UserId,
    is_bot_admin: bool,
    custom_tag: Option<String>,
}

impl Permissions {
    pub async fn set_bot_admin(&mut self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        self.is_bot_admin = true;

        // sqlx::query("INSERT OR IGNORE permissions (user_id, is_bot_admin) VALUES (?, 1)")
        //     .bind(&self.user_id.0.to_string())
        //     .bind(&self.is_bot_admin)
        //     .execute(pool)
        //     .await?;

        // sqlx::query("UPDATE permissions SET is_bot_admin = 1 WHERE user_id = ?")
        //     .bind(self.user_id)
        //     .execute(pool)
        //     .await?;

        Ok(())
    }
}