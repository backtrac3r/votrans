mod bot_state;
mod helpers;
mod markups;

use api::{Lang, Ytdlp};
use bot_state::Config;
use bytes::Bytes;
use futures::StreamExt;
use helpers::send_response_txt;
use reqwest::{header, multipart};
use std::sync::Arc;
use teloxide::{
    dispatching::dialogue::InMemStorage,
    net::Download,
    prelude::*,
    types::{MediaKind, MessageKind},
};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub const MSG_CH_LIMIT: usize = 4096;

#[derive(Clone)]
pub enum Data {
    Url(String),
    FileId(String),
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    HighlightWords {
        data: Data,
    },
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("env error (file .env not found)");

    let bot = Bot::from_env();

    let bot_state = Arc::new(Config::new());

    let mut test_bytes = Vec::<u8>::new();
    test_bytes.append(&mut vec![1, 2]);
    test_bytes.append(&mut vec![3, 4]);

    Dispatcher::builder(
        bot,
        dptree::entry().branch(
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<State>, State>()
                .branch(dptree::case![State::Start].endpoint(start_handler))
                .branch(dptree::case![State::HighlightWords { data }].endpoint(words_handler)),
        ),
    )
    .dependencies(dptree::deps![bot_state, InMemStorage::<State>::new()])
    .build()
    .dispatch()
    .await;
}

async fn start_handler(
    bot: Bot,
    dialogue: MyDialogue,
    app_data: Arc<Config>,
    msg: Message,
) -> HandlerResult {
    let chat_id = msg.chat.id;

    let Some(txt) = msg.text() else {
        let MessageKind::Common(common_msg) = &msg.kind else {
            bot.send_message(
                chat_id,
                "отправь мне ссылку на видео, голосовое сообщение, или видеофайл",
            )
            .await?;
            return Ok(());
        };

        bot.send_message(chat_id, "Начал обработку").await?;

        let file = match &common_msg.media_kind {
            MediaKind::Video(m) => {
                let file = bot.get_file(&m.video.file.id).await.unwrap();
                file
            }
            MediaKind::Voice(m) => {
                let file = bot.get_file(&m.voice.file.id).await.unwrap();
                file
            }
            MediaKind::Document(m) => {
                let file = bot.get_file(&m.document.file.id).await.unwrap();
                file
            }
            _ => {
                bot.send_message(
                    chat_id,
                    "отправь мне ссылку на видео, голосовое сообщение, или видеофайл",
                )
                .await?;
                return Ok(());
            }
        };

        let mut file_stream = bot.download_file_stream(&file.path);

        let mut file_bytes = Vec::<Bytes>::new();
        while let Some(Ok(b)) = file_stream.next().await {
            file_bytes.push(b);
        }
        let file_bytes = file_bytes.concat();

        let part = multipart::Part::stream(file_bytes).file_name(file.id.to_string());
        let form = multipart::Form::new().part("file", part);

        let mut headers = header::HeaderMap::new();
        headers.insert("accept", "application/json".parse().unwrap());

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

        send_response_txt(&response, &bot, &msg).await?;

        return Ok(());
    };

    if txt == "/start" {
        bot.send_message(
            chat_id,
            "Отправь мне ссылку на видео из VK/YouTube, а я переведу речь из видео в текст",
        )
        .await?;

        return Ok(());
    }

    if !(txt.contains("youtube.com") || txt.contains("youtu.be") || txt.contains("vk.com")) {
        bot.send_message(chat_id, "это не похоже на ссылку").await?;

        return Ok(());
    }

    let req_body = Ytdlp {
        url: txt.to_string(),
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

    send_response_txt(&response, &bot, &msg).await?;

    Ok(())
}
