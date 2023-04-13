use log::{debug, error, info};
use std::env;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use teloxide::types::Update;
use tokio::sync::mpsc;

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

    let (tx, rx) = mpsc::channel::<Box<Update>>(32);

    let token = get_env("TG_TOKEN")?;
    let timeout = get_env("TG_TIMEOUT").unwrap_or("10".to_string()).parse::<u32>()?;
    info!("Telegram long polling timeout has been set to {} seconds", timeout);
    let tg_bot = tg::TgBot::new(token).await?;

    let exit_condition = Arc::new(AtomicBool::new(false)); //TODO: use cancellation token instead
    futures_util::try_join!(
        get_err(exit_condition.clone()),
        tg_bot.process_messages(timeout, tx, exit_condition.clone()),
        process_messages(rx, db, exit_condition.clone()),
    )?;

    Ok(())
}

async fn process_messages(mut rx: mpsc::Receiver<Box<Update>>, db: db::Db, exit_condition: Arc<AtomicBool>) -> Result<(), Box<dyn Error>> {
    while !exit_condition.load(Ordering::SeqCst) {
        let update = rx.recv().await;
        match update {
            Some(update) => {
                debug!("Update: {:?}", update);
                db.save_update(update).await?;

            }
            None => {
                info!("Channel closed");
                break;
            }
        }
    }

    Ok(())
}

fn get_env(key: &str) -> Result<String, String> {
    env::var(key).map_err(|_| format!("Couldn't read environment variable '{}'", key))
}

async fn get_err(exit_trigger: Arc<AtomicBool>) -> Result<(), Box<dyn Error>> {
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    //Err(Box::try_from("BOOM".to_string()).unwrap())
    exit_trigger.store(true, Ordering::SeqCst);
    Ok(())
}