use teloxide::types::{KeyboardButton, KeyboardMarkup, ReplyMarkup};

fn kb_markup(keyboard: &[Vec<&str>]) -> ReplyMarkup {
    let kb: Vec<Vec<KeyboardButton>> = keyboard
        .iter()
        .map(|row| {
            let row: Vec<String> = row.iter().map(std::string::ToString::to_string).collect();
            row.iter().map(KeyboardButton::new).collect()
        })
        .collect();

    let markup = KeyboardMarkup::new(kb).resize_keyboard(true);

    ReplyMarkup::Keyboard(markup)
}
