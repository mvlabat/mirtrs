use dotenv;
use futures::Stream;
use telegram_bot::{
    AllowedUpdate, AnswerInlineQuery, Api, CanReplySendMessage, InlineKeyboardButton,
    InlineKeyboardMarkup, InlineQueryResultArticle, InputTextMessageContent, MessageKind,
    ParseMode, UpdateKind,
};
use tokio_core::reactor::Core;

use std::{
    collections::BTreeMap,
    env,
    sync::{Arc, Mutex},
    time::Instant,
};

fn main() {
    let mut core = Core::new().unwrap();

    dotenv::dotenv().unwrap();
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let api = Api::configure(token).build(core.handle()).unwrap();

    let mut stream = api.stream();
    stream.allowed_updates(&[
        AllowedUpdate::Message,
        AllowedUpdate::EditedMessage,
        AllowedUpdate::ChannelPost,
        AllowedUpdate::EditedChannelPost,
        AllowedUpdate::InlineQuery,
        AllowedUpdate::ChosenInlineResult,
        AllowedUpdate::CallbackQuery,
        AllowedUpdate::ShippingQuery,
        AllowedUpdate::PreCheckoutQuery,
    ]);

    let mut last_entries = BTreeMap::<Instant, String>::new();
    last_entries.insert(Instant::now(), "Test".to_owned());
    last_entries.insert(Instant::now(), "Wow".to_owned());
    last_entries.insert(Instant::now(), "Another test".to_owned());
    let last_entries = Arc::new(Mutex::new(last_entries));

    // Fetch new updates via long poll method
    let future = stream.for_each(|update| {
        // If the received update contains a new message...
        match update.kind {
            UpdateKind::Message(message) => {
                if let MessageKind::Text { ref data, .. } = message.kind {
                    // Print received text message to stdout.
                    println!("<{}>: {}", &message.from.first_name, data);

                    // Answer message with "Hi".
                    api.spawn(message.text_reply(format!(
                        "Hi, {}! You just wrote '{}'",
                        &message.from.first_name, data
                    )));
                }
            }
            UpdateKind::EditedMessage(_message) => {
                println!("EditedMessage");
            }
            UpdateKind::ChannelPost(_channel_post) => {
                println!("ChannelPost");
            }
            UpdateKind::EditedChannelPost(_channel_post) => {
                println!("EditedChannelPost");
            }
            UpdateKind::InlineQuery(inline_query) => {
                println!("{:?}", inline_query);

                let query = inline_query.query.trim();
                let answers = if query.ends_with("go") || query.starts_with("go ") {
                    let last_entries = last_entries.lock().expect("Failed to lock the mutex");
                    last_entries.iter().filter(|(_, last_entry)| {
                        query.len() < 3 || last_entry.starts_with(&query[3..])
                    }).map(|(_, last_entry)| {
                        let mut result = InlineQueryResultArticle::new(
                            last_entry.clone(),
                            last_entry.clone(),
                            InputTextMessageContent {
                                message_text: format!("Voting for {}", &last_entry),
                                parse_mode: None,
                                disable_web_page_preview: true,
                            },
                        );
                        let mut keyboard_markup = InlineKeyboardMarkup::new();
                        keyboard_markup.add_row(vec![
                            InlineKeyboardButton::callback("Yiss", "yes"),
                            InlineKeyboardButton::callback("Nope", "no"),
                            InlineKeyboardButton::callback("Dunno", "maybe"),
                        ]);
                        result.reply_markup(keyboard_markup);
                        result.into()
                    }).collect::<Vec<_>>()
                } else {
                    vec![
                        InlineQueryResultArticle::new(
                            "time",
                            "IT IS TIME",
                            InputTextMessageContent {
                                message_text: "TIS MUST BE THE TIME BUT I CAN'T CHECK YET. IS IT 11:11 OR SMTH???".to_owned(),
                                parse_mode: None,
                                disable_web_page_preview: true,
                            },
                        ).into(),
                        InlineQueryResultArticle::new(
                            "hello_world",
                            "HELLO HUMANS",
                            InputTextMessageContent {
                                message_text: "hello humans i'm a useless bot *beep boop beep* i can print messages...".to_owned(),
                                parse_mode: Some(ParseMode::Markdown),
                                disable_web_page_preview: true,
                            },
                        ).into(),
                    ]
                };

                api.spawn(AnswerInlineQuery::new(
                    inline_query.id,
                    answers,
                ));
            }
            UpdateKind::CallbackQuery(callback_query) => {
                println!("{:?}", callback_query);
            }
            UpdateKind::Error(_err_string) => {
                println!("Error");
            }
            UpdateKind::Unknown => {
                println!("Unknown");
            }
        }

        Ok(())
    });

    core.run(future).unwrap();
}
