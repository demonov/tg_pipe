use log::{debug, error, info};
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use teloxide::Bot;
use teloxide::payloads::GetUpdates;
use teloxide::prelude::*;
use teloxide::requests::JsonRequest;
use teloxide::types::AllowedUpdate::*;
use teloxide::types::Me;
use tokio::sync::mpsc::Sender;

pub struct TgBot {
    bot: Bot,
    me: Me,
}

impl TgBot {
    pub async fn new(token: String) -> Result<Self, Box<dyn Error>> {
        let bot = Bot::new(token);
        let me = bot.get_me().send().await?;
        info!("I am: {:?}", me.user);

        Ok(Self {
            bot,
            me,
        })
    }

    pub async fn process_messages(&self, timeout: u32, sender: Sender<Box<Update>>, exit_condition: Arc<AtomicBool>) -> Result<(), Box<dyn Error>> {
        let mut offset = None;
        while !exit_condition.load(Ordering::SeqCst) {
            let updates = prepare_update_request(&self.bot, timeout, offset);
            debug!("requesting updates with offset: {:?}", updates.offset);
            let mut updates = match updates.send().await {
                Ok(updates) => updates,
                Err(e) => {
                    error!("Error getting updates: {}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(timeout as u64)).await;
                    continue;
                }
            };

            updates.sort_by_key(|u| u.id);
            offset = updates.last().map_or(None, |u| Some(&u.id + 1));

            for update in updates {
                debug!("Tg update: {:?}", &update);
                sender.send(Box::new(update)).await?;
            }
        }

        Ok(())
    }
}

fn prepare_update_request(bot: &Bot, timeout: u32, offset: Option<i32>) -> JsonRequest<GetUpdates> {
    let mut request = bot
        .get_updates()
        .timeout(timeout)
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

    request.offset = offset;
    request
}


