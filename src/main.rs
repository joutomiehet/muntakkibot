use dotenv::dotenv;
use reqwest;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio;

mod commands;
mod helpers;
use commands::*;
use helpers::*;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenv().ok();
    log::info!("Starting bot");
    let bot: Bot = Bot::from_env();

    bot.set_my_commands(Command::bot_commands())
        .await
        .expect("Failed to set commands");

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        // Accept photos only if in a private chat

        if let Some(photo) = msg.photo()
            && msg.chat.is_private()
        {
            if let Some(last_photo) = photo.last() {
                let file_id: &String = &last_photo.file.id;
                let file: teloxide::types::File = bot.get_file(file_id).await?;
                let file_path: String = file.path;
                let file_url: String = format!(
                    "https://api.telegram.org/file/bot{}/{}",
                    bot.token(),
                    file_path
                );
                // Download photo
                let response: reqwest::Response = reqwest::get(&file_url).await?;
                let bytes = response.bytes().await?;

                if let Some(u) = msg.from {
                    let user = u;
                    let user_id: String = user.id.to_string();
                    let nickname: &str = user.username.as_deref().unwrap_or_default();

                    let filename =
                        Path::new(&*IMAGE_DIR).join(format!("takki_{}_{}.jpg", user_id, nickname));
                    let mut file: std::fs::File = File::create(&filename)?;
                    file.write_all(&bytes)?;

                    // In case old named file exists, remove it
                    let _ = fix_file_name(&user_id, nickname, &*IMAGE_DIR);

                    bot.send_message(msg.chat.id, "Kuva vastaanotettu ja tallennettu")
                        .await?;
                }
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
                        bot.send_message(msg.chat.id, "Unkown command. Try /help")
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
