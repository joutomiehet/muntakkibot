use dotenv::dotenv;
use regex::Regex;
use reqwest;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use teloxide::{prelude::*, types::InputFile, utils::command::BotCommands};
use tokio;
use std::env;
#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenv().ok();
    log::info!("Starting bot");

    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        let image_dir = env::var("IMAGE_DIR").expect("IMAGE_DIR not found");

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

                let user = msg.from.unwrap();
                let user_id = user.id.to_string();
                let nickname = user.username.as_deref().unwrap_or_default();

                let mut file = File::create(format!("{}/takki_{}_{}.jpg", image_dir, user_id, nickname))?;
                file.write_all(&bytes)?;

                // In case old named file exists, remove it
                fix_file_name(&user_id, nickname);

                bot.send_message(msg.chat.id, "Kuva vastaanotettu ja tallennettu")
                    .await?;
            }
        } else if let Some(text) = msg.text() {
            if let Ok(cmd) = Command::parse(text, "MunTakkiBot") {
                match cmd {
                    Command::MunTakki => {
                        let user = msg.from.as_ref().unwrap();
                        let user_id = user.id.to_string();
                        let nickname = user.username.as_deref().unwrap_or_default();
                        let name = &user.first_name;

                        fix_file_name(&user_id, nickname);

                        // Create regex to find takki by either ID or telegram nickname
                        // Try id first in case nickname has changed
                        let re = Regex::new(&format!(r"takki_({}|{})_.*\.jpg", user_id, name)).unwrap();

                        let _ = get_takki(&msg, &bot, re, &name).await;
                    }
                    
                    Command::SunTakki => {
                        let message: Vec<&str> = msg.text().unwrap().trim().split(" ").collect();

                        if message.len() < 2 {
                            bot.send_message(msg.chat.id, "Umm unohidtko laittaa nickin?").await?;
                        } else {
                            for nick in &message[1..] {
                                let nick_cleaned = nick.trim_start_matches("@");
                                let re = Regex::new(&format!(r"takki_.*_({}).jpg", nick_cleaned)).unwrap();
                                let _ = get_takki(&msg, &bot, re, nick).await;
                            }
                        }
                    }
                    Command::Help => {
                        let general_help = "Tällä botilla voit tallentaa kuvan omasta Joutotakistasi ja näyttää siitä helposti kuvan kun tarvitset jonkun hakemaan takkisi solusta.\nRiittää että lähetät kuvan takistasi yksityisviestillä niin se tallentuu. Voit päivittää kuvan lähettämällä uuden kuvan.";
                        let komennot = "Komennot:\n/muntakki -> Lähettää kuvan sinun takistasi\n/suntakki @<tg-nicki> -> Hae toisen käyttäjän takki. Voit myös laitta monta käyttäjää peräkkäin samaan komentoon.";
                        let descriptions = Command::descriptions().to_string();
                        bot.send_message(msg.chat.id, format!("{}\n{}\n{}", general_help, descriptions, komennot))
                            .await?;
                    }
                    Command::Start => {
                        let start = "Tervetuloa käyttämään MunTakkiBotia. Botti toimii hyvin yksinkertaisesti.\n1. Lähetä kuva takistasi minulle yksityisviestillä\n2. Käytä /muntakki komentoa chatissa.\n3. Käytä /suntakki <tg-nick> katsoaksesi jonkun muun takin.";
                        bot.send_message(msg.chat.id, start).await?;
                    }
                }
            }
        }
        Ok(())
    })
    .await;
}

async fn get_takki(
    msg: &Message,
    bot: &Bot,
    re: Regex,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let image_dir = env::var("IMAGE_DIR").expect("IMAGE_DIR not found");

    let photos = fs::read_dir(image_dir)?;

    // We need to loop through all the files
    let mut found_file = None;
    for photo in photos {
        let p = photo?;
        let file_name = p.file_name();
        let file_name = file_name.to_string_lossy();
        if re.is_match(&file_name) {
            found_file = Some(p.path());
            break;
        }
    }

    if let Some(file_path) = found_file {
        let file = InputFile::file(file_path);
        bot.send_photo(msg.chat.id, file).await?;
    } else {
        bot.send_message(msg.chat.id, format!("Takkiasi ei löytynyt {}", name))
            .await?;
    }
    Ok(())
}

fn fix_file_name(user_id: &str, nickname: &str) {
    // Check if theres possibly duplicate pictures
    let image_dir = env::var("IMAGE_DIR").expect("IMAGE_DIR not found");
    let old_filename = format!("{}/takki_{}.jpg", image_dir, user_id);
    let new_filename = format!("{}/takki_{}_{}.jpg", image_dir, user_id, nickname);

    let old_path = Path::new(&old_filename);
    let new_path = Path::new(&new_filename);

    // No old path? No problem!
    if !old_path.exists() {
        return
    }

    // Remove the old file
    if old_path.exists() && new_path.exists() {
        println!("Both exist!");
        match fs::remove_file(old_path) {
            Ok(_) => (),
            Err(e) => println!("Oops couldn't remove old file: {}", e),
        }
        return;
    }

    // Found just the old file. Need to rename it
    if old_path.exists() && !new_path.exists() {
        match fs::rename(&old_path, &new_path) {
            Ok(_) => (),
            Err(e) => println!("Failed to rename file: {}", e),
        }
    }

}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "")]
enum Command {
    #[command(description = "Lähettää kuvan takistasi")]
    MunTakki,
    #[command(description = "Lähettää kuvan kaverin takista")]
    SunTakki,
    #[command(description = "Näyttää tämän viestin")]
    Help,
    #[command(description = "Näyttää tervetuliaisviestin")]
    Start
}
