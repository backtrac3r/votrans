mod bot_state;
mod helpers;
mod markups;

use bot_state::Config;
use helpers::{file_tt, send_response_txt, url_tt};
use markups::{select_words_options, start_options};
use std::{collections::HashSet, sync::Arc};
use teloxide::{
    dispatching::dialogue::InMemStorage,
    prelude::*,
    types::{MediaKind, MessageKind},
};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), BotErr>;
type BotErr = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone)]
pub enum Data {
    Url(String),
    FileId(String),
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    SelectWords,
    WordsHighlight {
        words_list: HashSet<String>,
    },
    HighlightWords {
        data: Data,
    },
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
                .branch(dptree::case![State::Start].endpoint(start_handler))
                .branch(dptree::case![State::SelectWords].endpoint(select_words_handler))
                .branch(
                    dptree::case![State::WordsHighlight { words_list }]
                        .endpoint(words_highlight_handler),
                ),
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

        let response = file_tt(&file.path, &bot, &msg, &app_data).await?;

        send_response_txt(&response, &bot, &msg).await?;

        return Ok(());
    };

    match txt {
        "/select_words" | "Выбрать слова" => {
            dialogue.update(State::SelectWords).await?;
            bot.send_message(
                chat_id,
                "Скинь список слов, разделенных пробелом, которые я потом выделю в тексте",
            )
            .reply_markup(select_words_options())
            .await?;

            return Ok(());
        }
        "/start" => {
            bot.send_message(
                chat_id,
                "Отправь мне файл/гс/ссылку на видео из VK/YouTube, а я переведу речь из видео в текст",
            )
            .reply_markup(start_options())
            .await?;

            return Ok(());
        }
        _ => (),
    }

    let response = url_tt(txt, &bot, &msg, &app_data).await?;

    send_response_txt(&response, &bot, &msg).await?;

    Ok(())
}

async fn select_words_handler(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id;

    let Some(txt) = msg.text() else {
        bot.send_message(
            chat_id,
            "Скинь список слов, разделенных пробелом, которые я потом выделю в тексте",
        )
        .await?;

        return Ok(());
    };

    match txt {
        "Назад" => {
            dialogue.update(State::Start).await?;

            bot.send_message(
                chat_id,
                "Отправь мне ссылку на видео из VK/YouTube, а я переведу речь из видео в текст",
            )
            .await?;
        }
        _ => {
            let words_list: HashSet<String> = txt
                .split(' ')
                .map(|word| word.to_string().to_lowercase())
                .collect();

            bot.send_message(
                chat_id,
                "Отправь мне гс/видеофайл ссылку на видео из VK/YouTube, а я переведу речь из видео в текст и выделю твои слова",
            )
            .await?;

            dialogue
                .update(State::WordsHighlight { words_list })
                .await?;
        }
    }

    Ok(())
}

async fn words_highlight_handler(
    bot: Bot,
    dialogue: MyDialogue,
    app_data: Arc<Config>,
    words_list: HashSet<String>,
    msg: Message,
) -> HandlerResult {
    let chat_id = msg.chat.id;

    let response = if let Some(txt) = msg.text() {
        if txt == "Назад" {
            bot.send_message(
                chat_id,
                "Отправь мне ссылку на видео из VK/YouTube, а я переведу речь из видео в текст",
            )
            .reply_markup(start_options())
            .await?;

            return Ok(());
        }

        url_tt(txt, &bot, &msg, &app_data).await?
    } else {
        let MessageKind::Common(common_msg) = &msg.kind else {
            bot.send_message(
                chat_id,
                "отправь мне ссылку на видео, голосовое сообщение, или видеофайл",
            )
            .await?;

            return Ok(());
        };

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

        file_tt(&file.path, &bot, &msg, &app_data).await?
    };

    // highlight words
    let mut resp_words: Vec<String> = response.split(' ').map(|w| w.to_string()).collect();

    resp_words.iter_mut().for_each(|response_word| {
        if words_list.contains(&response_word.to_string().to_lowercase()) {
            *response_word = format!("__{response_word}__");
        }
    });

    let response = resp_words.join(" ");

    send_response_txt(&response, &bot, &msg).await?;

    bot.send_message(
        chat_id,
        "Отправь мне ссылку на видео из VK/YouTube, а я переведу речь из видео в текст и выделю твои слова",
    )
    .await?;

    dialogue
        .update(State::WordsHighlight { words_list })
        .await?;

    Ok(())
}
