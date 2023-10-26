mod bot_state;

use api::Ytdlp;
use bot_state::Config;
use bytes::Bytes;
use futures::StreamExt;
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

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
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
                .branch(dptree::case![State::Start].endpoint(start)),
        ),
    )
    .dependencies(dptree::deps![bot_state, InMemStorage::<State>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

async fn start(
    bot: Bot,
    _dialogue: MyDialogue,
    app_data: Arc<Config>,
    msg: Message,
) -> HandlerResult {
    let chat_id = msg.chat.id;

    let Some(txt) = msg.text() else {
        let MessageKind::Common(common_msg) = msg.kind else {
            bot.send_message(
                chat_id,
                "отправь мне ссылку на видео, голосовое сообщение, или видеофайл",
            )
            .await?;
            return Ok(());
        };

        bot.send_message(chat_id, "Начал обработку").await?;

        let file = match common_msg.media_kind {
            MediaKind::Video(v) => bot.get_file(&v.video.file.id).await.unwrap(),
            MediaKind::Voice(v) => bot.get_file(&v.voice.file.id).await.unwrap(),
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

        let part = multipart::Part::stream(file_bytes);
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

        bot.send_message(chat_id, response).await?;

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

    let url = format!("http://localhost:{}/url_tt", app_data.server_port);
    let resp = app_data
        .client
        .post(url)
        .json(&req_body)
        .send()
        .await?
        .text()
        .await?;

    bot.send_message(chat_id, resp).await?;

    Ok(())
}
