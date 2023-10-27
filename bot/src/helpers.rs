use crate::{HandlerResult, MSG_CH_LIMIT};
use teloxide::prelude::*;

pub async fn send_response_txt(resp: &str, bot: &Bot, chat_id: ChatId) -> HandlerResult {
    let msgs = resp
        .chars()
        .collect::<Vec<char>>()
        .chunks(MSG_CH_LIMIT)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<String>>();

    for msg in msgs {
        bot.send_message(chat_id, msg).await.unwrap();
    }

    Ok(())
}
