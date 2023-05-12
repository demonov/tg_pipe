use clap::Parser;
use log::{debug, error, info};
use std::env;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::{ChatId, Update};
use teloxide::types::UpdateKind;
use tokio::time::sleep;
use gpt::Gpt;
use crate::db::{ConfKey, Db};
use crate::gpt::ChatMessage;
use crate::tg::TgBot;

mod db;
mod gpt;
mod tg;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;
    env_logger::init();

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

#[derive(Parser)]
struct Cli {
    /// Set telegram Chat Id
    #[arg(short, value_name = "chat_id")]
    chat_id: Option<i64>,

    /// Set the user as bot admin
    #[arg(short('a'), value_name = "user_id")]
    set_bot_admin: Option<u64>,

    /// Reset telegram update offset
    #[arg(short('o'))]
    reset_offset: bool,
}

async fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    info!("Starting...");
    let db = get_env("DB")?;
    let db = Db::new(db).await?;
    db.migrate().await?;

    if cli.reset_offset {
        info!("Resetting telegram update offset...");
        db.write_conf_value::<String>(ConfKey::Offset, None).await?;
    }

    if let Some(chat_id) = cli.chat_id {
        info!("Setting telegram chat id to {}...", chat_id);
        db.write_conf_value(ConfKey::ChatId, Some(chat_id.to_string())).await?;
    }

    if let Some(user_id) = cli.set_bot_admin {
        info!("Setting user {} as bot admin...", user_id);
        db.set_bot_admin(user_id).await?;
    }

    let prompt = db.read_conf_value::<String>(ConfKey::GptPrompt).await?.ok_or("Prompt is not set")?;
    let history_capacity = 3; //db.read_conf_value::<usize>(ConfKey::HistoryCapacity).await?.ok_or("History capacity is not set")?;

    let openai_key = get_env("OPENAI_KEY")?;
    let gpt = Gpt::new(openai_key,"gpt-3.5-turbo-0301".to_string(), prompt, history_capacity)?; //TODO: make model configurable

    let token = get_env("TG_TOKEN")?;
    let tg_lp_timeout = get_env("TG_LONGPOOL_TIMEOUT").unwrap_or("10".to_string()).parse::<u32>()?;
    info!("Telegram long polling timeout has been set to {} seconds", tg_lp_timeout);
    let tg_retry_timeout = get_env("TG_RETRY_TIMEOUT").unwrap_or("5".to_string()).parse::<u64>()?;
    let tg_retry_timeout = Duration::from_secs(tg_retry_timeout);
    info!("Telegram retry timeout has been set to {} seconds", tg_retry_timeout.as_secs());
    let chat_id = db.read_conf_value::<String>(ConfKey::ChatId).await?.ok_or("Chat id is not set")?;
    let chat_id = chat_id.parse::<i64>()?;
    let tg_bot = TgBot::new(token, tg_lp_timeout, chat_id).await?;

    let exit_condition = Arc::new(AtomicBool::new(false)); //TODO: use cancellation token instead
    futures_util::try_join!(
        process_messages(tg_bot, db, gpt, exit_condition.clone(), tg_retry_timeout),
    )?;

    Ok(())
}

async fn process_messages(tg_bot: TgBot, db: Db, mut gpt:Gpt, exit_trigger: Arc<AtomicBool>, retry_timeout: Duration) -> Result<(), Box<dyn Error>> {
    let mut offset = db.read_conf_value(ConfKey::Offset).await?;

    while !exit_trigger.load(std::sync::atomic::Ordering::SeqCst) { //TODO: use cancellation token instead
        match tg_bot.get_updates(offset).await {
            Ok(updates) => {
                db.save_updates(&updates).await?;

                for update in &updates {
                    debug!("Update: {:?}", update);
                }

                let chat_updates = get_chat_updates(&updates, tg_bot.chat_id);
                if let Some(response) = gpt.query(chat_updates).await? {
                    debug!("Response: {}", response);
                    tg_bot.send_message(tg_bot.chat_id, &response).await?;
                }

                offset = updates.last().map_or(None, |u| Some(u.id + 1));
                db.write_conf_value(ConfKey::Offset, offset).await?;
            }
            Err(e) => {
                error!("error getting updates from tg: {:?}", e);
                sleep(retry_timeout).await;
            }
        }
    }

    info!("processing messages stopped");
    Ok(())
}


fn get_chat_updates(tg_updates: &Vec<Update>, chat_id: ChatId) -> Vec<ChatMessage> {
    tg_updates.iter()
        .filter(|u| u.chat_id() == Some(chat_id))
        .filter_map(|u| match &u.kind {
            UpdateKind::Message(m) => Some(m.into()),
            _ => None})
        .collect::<Vec<_>>()
}

fn get_env(key: &str) -> Result<String, String> {
    env::var(key).map_err(|_| format!("Couldn't read environment variable '{}'", key))
}

