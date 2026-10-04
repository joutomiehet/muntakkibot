use dotenv::dotenv;
use std::fs;
use teloxide::{net::Download, prelude::*, utils::command::BotCommands};

mod commands;
mod helpers;
use commands::*;
use helpers::*;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenv().ok();
    log::info!("Starting bot");
    fs::create_dir_all(&*IMAGE_DIR).expect("Failed to create image directory");
    let bot: Bot = Bot::from_env();

    bot.set_my_commands(Command::bot_commands())
        .await
        .expect("Failed to set commands");

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        // Accept photos only if in a private chat

        if let Some(photo) = msg.photo()
            && msg.chat.is_private()
        {
            let Some(user) = &msg.from else {
                return Ok(());
            };
            if let Some(last_photo) = photo.last() {
                let file_id: &String = &last_photo.file.id;
                let file: teloxide::types::File = bot.get_file(file_id).await?;

                // Download photo
                let mut bytes: Vec<u8> = Vec::new();
                bot.download_file(&file.path, &mut bytes).await?;

                let nickname: &str = user.username.as_deref().unwrap_or_default();
                save_takki(&IMAGE_DIR, &user.id.to_string(), nickname, &bytes)?;

                bot.send_message(msg.chat.id, "Kuva vastaanotettu ja tallennettu")
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "Kuvan lataus epäonnistui")
                    .await?;
            }
        } else if let Some(text) = msg.text() {
            match Command::parse(text, "MunTakkiBot") {
                Ok(cmd) => match cmd {
                    Command::MunTakki => mun_takki(&bot, &msg).await?,
                    Command::SunTakki => sun_takki(&bot, &msg).await?,
                    Command::Help => help(&bot, &msg).await?,
                    Command::Start => start(&bot, &msg).await?,
                },
                Err(_) => {
                    if msg.chat.is_private() {
                        bot.send_message(msg.chat.id, "Unknown command. Try /help")
                            .await?;
                    }
                    return Ok(());
                }
            }
        }
        Ok(())
    })
    .await;
}
