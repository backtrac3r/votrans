use crate::{bot_state::Config, BotErr, HandlerResult};
use api::Ytdlp;
use bytes::Bytes;
use futures::StreamExt;
use reqwest::{header, multipart};
use teloxide::{
    net::Download,
    prelude::*,
    types::{MediaKind, MessageKind, ParseMode},
};

pub const MSG_CH_LIMIT: usize = 4096;

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
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
    }

    Ok(())
}

pub async fn file_tt(
    file_path: &str,
    bot: &Bot,
    msg: &Message,
    app_data: &Config,
) -> Result<String, BotErr> {
    let chat_id = msg.chat.id;

    let mut file_stream = bot.download_file_stream(&file_path);

    let mut file_bytes = Vec::<Bytes>::new();
    while let Some(Ok(b)) = file_stream.next().await {
        file_bytes.push(b);
    }
    let file_bytes = file_bytes.concat();

    let part = multipart::Part::stream(file_bytes).file_name("file");
    let form = multipart::Form::new().part("file", part);

    let mut headers = header::HeaderMap::new();
    headers.insert("accept", "application/json".parse().unwrap());

    bot.send_message(chat_id, "Начал обработку").await?;

    let url = format!("http://localhost:{}/file_tt", app_data.server_port);
    let response = app_data
        .client
        .post(url)
        .headers(headers.clone())
        .multipart(form)
        .send()
        .await?
        .text()
        .await?;

    Ok(response)
}

pub async fn url_tt(
    url: &str,
    bot: &Bot,
    msg: &Message,
    app_data: &Config,
) -> Result<String, BotErr> {
    let chat_id = msg.chat.id;

    if !(url.contains("youtube.com") || url.contains("youtu.be") || url.contains("vk.com")) {
        bot.send_message(chat_id, "это не похоже на ссылку").await?;

        return Err(BotErr::from(""));
    }

    let req_body = Ytdlp {
        url: url.to_string(),
    };

    bot.send_message(chat_id, "Начал обработку").await?;

    let url = format!("http://127.0.0.1:{}/url_tt", app_data.server_port);
    let response = app_data
        .client
        .post(url)
        .json(&req_body)
        .send()
        .await?
        .text()
        .await?;

    Ok(response)
}

pub fn txt_screening(txt: &str) -> String {
    txt.replace('_', "\\_")
        .replace('*', "\\*")
        .replace(',', "\\,")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\)")
        .replace('~', "\\~")
        .replace('`', "\\`")
        .replace('>', "\\>")
        .replace('#', "\\#")
        .replace('+', "\\+")
        .replace('-', "\\-")
        .replace('=', "\\=")
        .replace('|', "\\|")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('.', "\\.")
        .replace('!', "\\!")
}

pub async fn get_file_from_msg(bot: &Bot, msg: &Message) -> Result<teloxide::types::File, BotErr> {
    let chat_id = msg.chat.id;

    let MessageKind::Common(common_msg) = &msg.kind else {
        bot.send_message(
            chat_id,
            "Отправь мне гс/кружок/аудиофайл/видеофайл/ссылку на видео из VK/YouTube",
        )
        .await?;

        return Err(BotErr::from(""));
    };

    let file = match &common_msg.media_kind {
        MediaKind::Video(m) => {
            let file = bot.get_file(&m.video.file.id).await?;
            file
        }
        MediaKind::Voice(m) => {
            let file = bot.get_file(&m.voice.file.id).await?;
            file
        }
        MediaKind::Document(m) => {
            let file = bot.get_file(&m.document.file.id).await?;
            file
        }
        MediaKind::Audio(m) => {
            let file = bot.get_file(&m.audio.file.id).await?;
            file
        }
        MediaKind::VideoNote(m) => {
            let file = bot.get_file(&m.video_note.file.id).await?;
            file
        }
        _ => {
            bot.send_message(
                chat_id,
                "Отправь мне гс/кружок/аудиофайл/видеофайл/ссылку на видео из VK/YouTube",
            )
            .await?;

            return Err(BotErr::from(""));
        }
    };

    Ok(file)
}
