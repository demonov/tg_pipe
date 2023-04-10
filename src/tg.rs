use std::error::Error;
use log::{debug, info};
use teloxide::Bot;
use teloxide::payloads::GetUpdates;
use teloxide::prelude::*;
use teloxide::requests::JsonRequest;
use teloxide::types::AllowedUpdate::*;
use teloxide::types::Me;

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

    pub async fn process_messages(&self, timeout: u32) -> Result<(), Box<dyn Error>> {
        let mut offset = None;
        loop {
            let updates = prepare_update_request(&self.bot, timeout, offset);
            debug!("requesting updates with offset: {:?}, timeout: {}", updates.offset, timeout);
            let mut updates = updates.send().await?;

            updates.sort_by_key(|u| u.id);
            offset = updates.last().map_or(None, |u| Some(&u.id + 1));

            for update in updates {
                debug!("Tg update: {:?}", &update);
            }
        }
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


