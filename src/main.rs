use log::{debug, error, info};
use std::env;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc;
use crate::tg::TgUpdate;

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

    let (tx, rx) = mpsc::channel::<TgUpdate>(1);

    let token = get_env("TG_TOKEN")?;
    let timeout = get_env("TG_TIMEOUT").unwrap_or("10".to_string()).parse::<u32>()?;
    info!("Telegram long polling timeout has been set to {} seconds", timeout);
    let tg_bot = tg::TgBot::new(token).await?;

    let exit_condition = Arc::new(AtomicBool::new(false)); //TODO: use cancellation token instead
    futures_util::try_join!(
        tg_bot.pull_updates(timeout, tx, exit_condition.clone()),
        process_messages(rx, db, exit_condition.clone()),
    )?;

    Ok(())
}

async fn process_messages(mut rx: mpsc::Receiver<TgUpdate>, db: db::Db, exit_trigger: Arc<AtomicBool>) -> Result<(), Box<dyn Error>> {
    loop {
        match rx.recv().await {
            Some(tg_update) => {
                db.save_updates(&tg_update.updates).await?;

                for update in &tg_update.updates {
                    debug!("Update: {:?}", update);
                }

                tg_update.done_processing();
            }
            None => {
                info!("Channel closed");
                break;
            }
        }
    }

    info!("processing messages stopped");
    Ok(())
}

fn get_env(key: &str) -> Result<String, String> {
    env::var(key).map_err(|_| format!("Couldn't read environment variable '{}'", key))
}
