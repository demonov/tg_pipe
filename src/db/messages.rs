use sqlx::{FromRow, SqlitePool};
use sqlx::sqlite::SqliteQueryResult;
use teloxide::prelude::{ChatId, UserId};
use teloxide::types::{Message as TgMessage, MessageId, Update, UpdateKind};

#[derive(Debug, FromRow)]
pub struct Message {
    update_id: i32,
    chat_id: Option<ChatId>,
    message_id: Option<MessageId>,
    kind: String,
    from_id: Option<UserId>,
    content: Option<String>,
    raw: Option<String>,
}

impl Message {
    pub fn from(update: &Update) -> Self {
        let (chat_id, message_id, from_id, content) = match &update.kind {
            UpdateKind::Message(message) => parse_message(&message),
            UpdateKind::EditedMessage(message) => parse_message(&message),
            UpdateKind::ChannelPost(message) => parse_message(&message),
            UpdateKind::EditedChannelPost(message) => parse_message(&message),
            UpdateKind::InlineQuery(q) => parse_unknown(q),
            UpdateKind::ChosenInlineResult(r) => parse_unknown(r),
            UpdateKind::CallbackQuery(q) => parse_unknown(q),
            UpdateKind::ShippingQuery(q) => parse_unknown(q),
            UpdateKind::PreCheckoutQuery(q) => parse_unknown(q),
            UpdateKind::Poll(p) => parse_unknown(p),
            UpdateKind::PollAnswer(pa) => parse_unknown(pa),
            UpdateKind::MyChatMember(m) => parse_unknown(m),
            UpdateKind::ChatMember(m) => parse_unknown(m),
            UpdateKind::ChatJoinRequest(r) => parse_unknown(r),
            UpdateKind::Error(e) => parse_unknown(e),
        };

        Self {
            update_id: update.id,
            chat_id,
            message_id,
            kind: upd_kind_to_string(&update.kind).to_string(),
            from_id,
            content,
            raw: get_raw(&update),
        }
    }

    pub async fn create_table(pool: &SqlitePool) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query(
                "CREATE TABLE IF NOT EXISTS 'messages' ( \
                    'update_id' INTEGER UNIQUE, \
                    'kind' TEXT NOT NULL, \
                    'chat_id' INTEGER, \
                    'message_id' INTEGER, \
                    'from_id' TEXT, \
                    'content' TEXT, \
                    'raw' TEXT, \
                    PRIMARY KEY('update_id') \
                );")
            .execute(pool)
            .await
    }

    pub async fn insert(&self, pool: &SqlitePool) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query(
            "INSERT INTO messages (update_id, kind, chat_id, message_id, from_id, content, raw) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(&self.update_id)
            .bind(&self.kind)
            .bind(&self.chat_id.map_or(None, |chat_id| Some(chat_id.0.to_string())))
            .bind(&self.message_id.map_or(None, |message_id| Some(message_id.0)))
            .bind(&self.from_id.map_or(None, |user_id| Some(user_id.0.to_string())))
            .bind(&self.content)
            .bind(&self.raw)
            .execute(pool)
            .await
    }
}

fn parse_message(msg: &TgMessage) -> (Option<ChatId>, Option<MessageId>, Option<UserId>, Option<String>) {
    let chat_id = Some(msg.chat.id);
    let message_id = Some(msg.id);
    let from_id = msg.from().map_or(None, |user| Some(user.id));
    let content = msg.text().map_or(None, |text| Some(text.to_string()));

    (chat_id, message_id, from_id, content)
}

fn parse_unknown<T>(_: T) -> (Option<ChatId>, Option<MessageId>, Option<UserId>, Option<String>) {
    (None, None, None, None)
}

fn upd_kind_to_string(kind: &UpdateKind) -> &'static str {
    match kind {
        UpdateKind::Message(_) => "Message",
        UpdateKind::EditedMessage(_) => "EditedMessage",
        UpdateKind::ChannelPost(_) => "ChannelPost",
        UpdateKind::EditedChannelPost(_) => "EditedChannelPost",
        UpdateKind::InlineQuery(_) => "InlineQuery",
        UpdateKind::ChosenInlineResult(_) => "ChosenInlineResult",
        UpdateKind::CallbackQuery(_) => "CallbackQuery",
        UpdateKind::ShippingQuery(_) => "ShippingQuery",
        UpdateKind::PreCheckoutQuery(_) => "PreCheckoutQuery",
        UpdateKind::Poll(_) => "Poll",
        UpdateKind::PollAnswer(_) => "PollAnswer",
        UpdateKind::MyChatMember(_) => "MyChatMember",
        UpdateKind::ChatMember(_) => "ChatMember",
        UpdateKind::ChatJoinRequest(_) => "ChatJoinRequest",
        UpdateKind::Error(_) => "Error",
    }
}

fn get_raw(u: &Update) -> Option<String> {
    match log::log_enabled!(log::Level::Debug) {
        true => Some(serde_json::to_string(u).unwrap_or("Error!".to_string())),
        false => None,
    }
}

