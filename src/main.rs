use std::cmp::max;
use std::env;
use std::error::Error;

use futures_util::{SinkExt, TryStreamExt};
use log::{debug, error, info, warn};
use teloxide::prelude::*;
use teloxide::types::AllowedUpdate::*;

mod db;
mod gpt;
mod tg;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;
    env_logger::init();
    info!("Starting...");

    match run().await {
        Ok(_) => {
            info!("Exit");
            Ok(())
        }
        Err(e) => {
            error!("Stopped with error: {:?}", e);
            Err(e)
        }
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    let db = get_env("DB")?;
    let db = db::Db::new(db).await?;
    db.migrate().await?;

    let openai_key = get_env("OPENAI_KEY")?;
    let gpt = gpt::Gpt::new("gpt-3.5-turbo".to_string(), openai_key)?;
    //let gpt = gpt::Gpt::new("gpt-4".to_string(), openai_key)?;
    let response = gpt.query().await?;
    info!("response: {}", response);

    let token = get_env("TG_TOKEN")?;
    let tgBot = tg::TgBot::new(token).await?;


    let mut offset = None;
    loop {
        let mut updates = bot.get_updates()
            .timeout(5)
            .allowed_updates(vec![Message,
                                  EditedMessage,
                                  ChannelPost,
                                  EditedChannelPost,
                                  InlineQuery,
                                  ChosenInlineResult,
                                  CallbackQuery,
                                  ShippingQuery,
                                  PreCheckoutQuery,
                                  Poll,
                                  PollAnswer,
                                  MyChatMember,
                                  ChatMember,
                                  ChatJoinRequest,
            ]);

        updates.offset = offset;

        debug!("requesting updates with offset: {:?}", updates.offset);
        let mut new_offset = None;
        for update in updates.send().await? {
            dbg!(&update);

            new_offset = new_offset.map_or(
                Some(update.id + 1),
                |o| Some(max(o, update.id + 1)));
        }
        offset = new_offset;
    }


    Ok(())
}

fn get_env(key: &str) -> Result<String, String> {
    env::var(key).map_err(|_| format!("Couldn't read environment variable '{}'", key))
}