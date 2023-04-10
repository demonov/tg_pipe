use std::env;
use std::error::Error;
use log::{error, info};

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
    let timeout = get_env("TG_TIMEOUT").unwrap_or("10".to_string()).parse::<u32>()?;
    let tg_bot = tg::TgBot::new(token).await?;

    tg_bot.process_messages(timeout).await?;

    Ok(())
}

fn get_env(key: &str) -> Result<String, String> {
    env::var(key).map_err(|_| format!("Couldn't read environment variable '{}'", key))
}