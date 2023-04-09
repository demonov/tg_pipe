use log::info;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::Me;

pub struct TgBot {
    bot: Bot,
    me: Me,
}

impl TgBot {
 pub async fn new(token: String) -> Self {
     let bot = Bot::new(token);
     let me = bot.get_me().send().await?;
     info!("I am: {:?}", me.user);
        Self {
            bot,
            me,
        }
    }
}