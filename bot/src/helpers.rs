use crate::{HandlerResult, MSG_CH_LIMIT};
use teloxide::prelude::*;

pub async fn send_response_txt(resp: &str, bot: &Bot, chat_msg: &Message) -> HandlerResult {
    let chat_id = chat_msg.chat.id;

    let msgs = resp
        .chars()
        .collect::<Vec<char>>()
        .chunks(MSG_CH_LIMIT)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<String>>();

    for msg in msgs {
        bot.send_message(chat_id, msg)
            .reply_to_message_id(chat_msg.id)
            .await?;
    }

    Ok(())
}
