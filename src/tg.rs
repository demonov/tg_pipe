use log::{debug, error, info};
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use teloxide::payloads::GetUpdates;
use teloxide::prelude::*;
use teloxide::requests::JsonRequest;
use teloxide::types::AllowedUpdate::*;
use teloxide::types::Me;
use tokio::sync::{mpsc, oneshot};

pub struct TgBot {
    bot: Bot,
    me: Me,
}

impl TgBot {
    pub async fn new<'a>(token: String) -> Result<Self, Box<dyn Error>> {
        let bot = Bot::new(token);
        let me = bot.get_me().send().await?;
        info!("I am: {:?}", me.user);

        Ok(Self {
            bot,
            me,
        })
    }

    pub async fn pull_updates(&self, timeout: u32, sender: mpsc::Sender<TgUpdate>, exit_condition: Arc<AtomicBool>) -> Result<(), Box<dyn Error>> {
        let mut offset = None;
        while !exit_condition.load(Ordering::SeqCst) {
            let updates = prepare_update_request(&self.bot, timeout, offset);
            debug!("requesting updates with offset: {:?}", updates.offset);
            let updates = match updates.send().await {
                Ok(updates) => updates,
                Err(e) => {
                    error!("Error getting updates: {}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(timeout as u64)).await;
                    continue;
                }
            };

            let (update, rx) = TgUpdate::new(updates);
            sender.send(update).await?;
            offset = rx.await?;
        }

        info!("tg::pull_updates stopped");
        Ok(())
    }
}

#[derive(Debug)]
pub struct TgUpdate {
    pub updates: Vec<Update>,
    new_offset_sender: oneshot::Sender<Option<i32>>,
    new_offset_value: Option<i32>,
}

impl TgUpdate {
    pub fn new(mut updates: Vec<Update>) -> (Self, oneshot::Receiver<Option<i32>>) {
        updates.sort_by_key(|u| u.id); // TODO: is this necessary?
        let new_offset_value = updates.last().map_or(None, |u| Some(u.id + 1));
        let (tx, rx) = oneshot::channel::<Option<i32>>();

        (Self {
            updates,
            new_offset_sender: tx,
            new_offset_value,
        }, rx)
    }

    pub fn done_processing(mut self) {
        match self.new_offset_sender.send(self.new_offset_value) {
            Ok(_) => (),
            Err(e) => error!("Error sending new offset: {:?}", e),
        }
    }
}

fn prepare_update_request(bot: &Bot, timeout: u32, offset: Option<i32>) -> JsonRequest<GetUpdates> {
    let mut request = bot.get_updates().timeout(timeout).allowed_updates(vec![Message,
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


