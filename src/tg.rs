use log::{debug, info};
use std::error::Error;
use teloxide::payloads::GetUpdates;
use teloxide::prelude::*;
use teloxide::requests::JsonRequest;
use teloxide::types::AllowedUpdate::*;

pub struct TgBot {
    lp_timeout: u32,
    bot: Bot,
}

impl TgBot {
    pub async fn new<'a>(token: String, lp_timeout: u32) -> Result<Self, Box<dyn Error>> {
        let bot = Bot::new(token);
        let me = bot.get_me().send().await?;
        info!("I am: {:?}", me.user);

        Ok(Self {
            bot,
            lp_timeout,
        })
    }

    pub async fn send_message(&self, chat_id: ChatId, message: &String) -> ResponseResult<teloxide::prelude::Message> {
        self.bot.send_message(chat_id, message).send().await
    }

    pub async fn get_updates(&self, offset: Option<i32>) -> ResponseResult<Vec<Update>> {
        let request = prepare_update_request(&self.bot, self.lp_timeout, offset);
        debug!("requesting updates with offset: {:?}", request.offset);

        request.send().await
    }
}

fn prepare_update_request(bot: &Bot, lp_timeout: u32, offset: Option<i32>) -> JsonRequest<GetUpdates> {
    let mut request = bot.get_updates().timeout(lp_timeout).allowed_updates(vec![Message,
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


