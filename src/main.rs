use dotenv::dotenv;
use reqwest;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use teloxide::{prelude::*, types::InputFile, utils::command::BotCommands};
use tokio;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenv().ok();
    log::info!("Starting bot");

    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        // Accept photos only if in a private chat
        if let Some(photo) = msg.photo() {
            if msg.chat.is_private() {
                let file_id = &photo.last().unwrap().file.id;
                let file = bot.get_file(file_id).await?;
                let file_path = file.path;
                let file_url = format!(
                    "https://api.telegram.org/file/bot{}/{}",
                    bot.token(),
                    file_path
                );
                // Download photo
                let response = reqwest::get(&file_url).await?;
                let bytes = response.bytes().await?;
                let mut file = File::create(format!("/usr/src/muntakkibot/images/takki_{}.jpg", msg.from.unwrap().id))?;
                file.write_all(&bytes)?;
                bot.send_message(msg.chat.id, "Photo received and saved!")
                    .await?;
            }
        } else if let Some(text) = msg.text() {
            if let Ok(cmd) = Command::parse(text, "MunTakkiBot") {
                match cmd {
                    Command::MunTakki => {
                        let filename = format!("/usr/src/muntakkibot/images/takki_{}.jpg", msg.from.as_ref().unwrap().id);
                        if Path::new(&filename).exists() {
                            let file = InputFile::file(filename);
                            bot.send_photo(msg.chat.id, file).await?;
                        } else {
                            bot.send_message(
                                msg.chat.id,
                                format!("Takkiasi ei löytynyt {}", msg.from.unwrap().first_name),
                            )
                            .await?;
                        }
                    }
                    Command::Help => {
                        let general_help = "Tällä botilla voit tallentaa kuvan omasta Joutotakistasi ja näyttää siitä helposti kuvan kun tarvitset jonkun hakemaan takkisi solusta. Riittää että lähetät kuvan takistasi yksityisviestillä niin se tallentuu.";
                        let descriptions = Command::descriptions().to_string();
                        bot.send_message(msg.chat.id, format!("{}\n{}", general_help, descriptions))
                            .await?;
                    }
                }
            }
        }
        Ok(())
    })
    .await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "")]
enum Command {
    #[command(description = "Lähettää kuvan takistasi")]
    MunTakki,
    #[command(description = "Näyttää tämän viestin")]
    Help,
}
