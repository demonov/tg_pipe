use log::{debug, error, info};
use std::env;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use tokio::time::sleep;
use crate::db::{ConfKey, Db};
use crate::tg::TgBot;

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

    /*
    let openai_key = get_env("OPENAI_KEY")?;
    let gpt = gpt::Gpt::new("gpt-3.5-turbo".to_string(), openai_key)?;
    //let gpt = gpt::Gpt::new("gpt-4".to_string(), openai_key)?;
    let response = gpt.query().await?;
    info!("response: {}", response);
    */

    let token = get_env("TG_TOKEN")?;
    let tg_lp_timeout = get_env("TG_LONGPOOL_TIMEOUT").unwrap_or("10".to_string()).parse::<u32>()?;
    info!("Telegram long polling timeout has been set to {} seconds", tg_lp_timeout);
    let tg_retry_timeout = get_env("TG_RETRY_TIMEOUT").unwrap_or("5".to_string()).parse::<u64>()?;
    let tg_retry_timeout = Duration::from_secs(tg_retry_timeout);
    info!("Telegram retry timeout has been set to {} seconds", tg_retry_timeout.as_secs());
    let tg_bot = TgBot::new(token, tg_lp_timeout).await?;

    let exit_condition = Arc::new(AtomicBool::new(false)); //TODO: use cancellation token instead
    futures_util::try_join!(
        process_messages(tg_bot, db, exit_condition.clone(), tg_retry_timeout),
    )?;

    Ok(())
}

async fn process_messages(tg_bot: TgBot, db: Db, exit_trigger: Arc<AtomicBool>, retry_timeout: Duration) -> Result<(), Box<dyn Error>> {
    let mut offset = db.read_conf_value(ConfKey::Offset).await?;

    while !exit_trigger.load(std::sync::atomic::Ordering::SeqCst) { //TODO: use cancellation token instead
        match tg_bot.get_updates(offset).await {
            Ok(updates) => {
                db.save_updates(&updates).await?;

                for update in &updates {
                    debug!("Update: {:?}", update);
                }

                offset = updates.last().map_or(None, |u|Some(u.id + 1));
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

fn get_env(key: &str) -> Result<String, String> {
    env::var(key).map_err(|_| format!("Couldn't read environment variable '{}'", key))
}
