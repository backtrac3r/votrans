mod bot_state;
mod helpers;

use bot_state::Config;
use helpers::Ytdlp;
use std::sync::Arc;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

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

async fn start(bot: Bot, _dialogue: MyDialogue, data: Arc<Config>, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id;

    let Some(txt) = msg.text() else {
        bot.send_message(chat_id, "Нужно отправить текст").await?;
        return Ok(());
    };

    if txt == "/start" {
        bot.send_message(
            chat_id,
            "Отправь мне ссылку на видео из VK/YouTube, а я переведу речь из видео в текст",
        )
        .await?;
    }

    let url = format!("http://localhost:{}/full", data.server_port);
    let req_body = Ytdlp {
        url: txt.to_string(),
    };

    let resp = data
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
