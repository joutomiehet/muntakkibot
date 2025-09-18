use dotenv::dotenv;
use regex::Regex;
use reqwest;
use std::env;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use teloxide::{prelude::*, types::*, utils::command::BotCommands};
use tokio;
use once_cell::sync::Lazy;

static IMAGE_DIR: Lazy<String> = Lazy::new(|| {
    env::var("IMAGE_DIR").unwrap()
});


#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenv().ok();
    log::info!("Starting bot");
    let bot: Bot = Bot::from_env();

    bot.set_my_commands(Command::bot_commands()).await.expect("Failed to set commands");

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {

        // Accept photos only if in a private chat
        if let Some(photo) = msg.photo() && msg.chat.is_private() {
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
                let bytes= response.bytes().await?;

                if let Some(u) = msg.from {
                    let user = u;
                    let user_id: String = user.id.to_string();
                    let nickname: &str = user.username.as_deref().unwrap_or_default();

                    let filename = Path::new(&*IMAGE_DIR).join(format!("takki_{}_{}.jpg", user_id, nickname));
                    let mut file: std::fs::File = File::create(&filename)?;
                    file.write_all(&bytes)?;

                    // In case old named file exists, remove it
                    let _ = fix_file_name(&user_id, nickname);

                    bot.send_message(msg.chat.id, "Kuva vastaanotettu ja tallennettu")
                        .await?;
                }
                
            } else {
                bot.send_message(msg.chat.id, "Kuvan lataus epäonnistui").await?;
            }
                
        } else if let Some(text) = msg.text() {
            if let Ok(cmd) = Command::parse(text, "MunTakkiBot") {
                match cmd {
                    Command::MunTakki => {
                        if let Some(user) = &msg.from {
                            if let Some(nickname) = &user.username {
                                let user_id = user.id.to_string();
                                let _ = fix_file_name(&user_id, nickname);

                                // Create regex to find takki by either ID or telegram nickname
                                // Try id first in case nickname has changed
                                let re: Regex = match Regex::new(&format!(r"(?i)takki_({}|{})_.*\.jpg", user_id, nickname)) {
                                    Ok(r) => r,
                                    Err(_) => {
                                        bot.send_message(msg.chat.id, "Error in Takki regex").await?;
                                        return Ok(());
                                    }
                                };

                                let _ = get_takki(&msg, &bot, re, &nickname).await;
                            }   
                        }
                    }

                    Command::SunTakki => {
                        let message: Vec<&str> = match msg.text() {
                            Some(text) => text.trim().split(" ").collect(),
                            None => Vec::new()
                        };

                        if message.len() < 2 {
                            bot.send_message(msg.chat.id, "Umm unohditko laittaa nickin?").await?;
                        } else {
                            for nick in &message[1..] {
                                let nick_cleaned: &str = nick.trim_start_matches("@");
                                let _ = match Regex::new(&format!(r"(?i)takki_.*_({}).jpg", nick_cleaned)) {
                                    Ok(re) => get_takki(&msg, &bot, re, nick).await,
                                    Err(_) => {
                                        println!("Regex error for nickname: {}", nick_cleaned);
                                        Ok(())
                                    }
                                };
                            }
                        }
                    }
                    Command::Help => {
                        let general_help: &str = "Tällä botilla voit tallentaa kuvan omasta Joutotakistasi ja näyttää siitä helposti kuvan kun tarvitset jonkun hakemaan takkisi solusta.\nRiittää että lähetät kuvan takistasi yksityisviestillä niin se tallentuu. Voit päivittää kuvan lähettämällä uuden kuvan.";
                        let komennot: &str = "Komennot:\n/muntakki -> Lähettää kuvan sinun takistasi\n/suntakki @<tg-nicki> -> Hae toisen käyttäjän takki. Voit myös laitta monta käyttäjää peräkkäin samaan komentoon.";
                        let descriptions: String = Command::descriptions().to_string();
                        bot.send_message(msg.chat.id, format!("{}\n{}\n{}", general_help, descriptions, komennot))
                            .await?;
                    }
                    Command::Start => {
                        let start: &str = "Tervetuloa käyttämään MunTakkiBotia. Botti toimii hyvin yksinkertaisesti.\n1. Lähetä kuva takistasi minulle yksityisviestillä\n2. Käytä /muntakki komentoa chatissa.\n3. Käytä /suntakki <tg-nick> katsoaksesi jonkun muun takin.";
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

    // We need to loop through all the files
    let found_photo = fs::read_dir(&*IMAGE_DIR)?
        .filter_map(Result::ok)
        .find(|entry| re.is_match(&entry.file_name().to_string_lossy()) )
        .map(|e| e.path());

    if let Some(file_path) = found_photo {
        let file = InputFile::file(file_path);
        bot.send_photo(msg.chat.id, file).await?;
    } else {
        bot.send_message(msg.chat.id, format!("Takkiasi ei löytynyt {}", name))
            .await?;
    }
    Ok(())
}

fn fix_file_name(user_id: &str, nickname: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Check if there's possibly duplicate pictures
    let old_path = Path::new(&*IMAGE_DIR).join(format!("takki_{}.jpg", user_id));
    let new_path = Path::new(&*IMAGE_DIR).join(format!("takki_{}_{}.jpg", user_id, nickname));


    // Remove the old file
    if old_path.exists() && new_path.exists() {
        println!("Both exist!");
        match fs::remove_file(old_path) {
            Ok(_) => (),
            Err(e) => println!("Oops couldn't remove old file: {}", e),
        }
        return Ok(());
    }

    // Found just the old file. Need to rename it
    if old_path.exists() && !new_path.exists() {
        match fs::rename(&old_path, &new_path) {
            Ok(_) => (),
            Err(e) => println!("Failed to rename file: {}", e),
        }
    }

    // Nickname has changed?
    let re: Regex = match Regex::new(&format!(r"(?i)takki_{}_([^_]+)\.jpg", user_id)) {
        Ok(r) => r,
        Err(e) => {
            println!("Failed to create regex: {}", e);
            return Ok(())
        }
    };
    let photos: crate::fs::ReadDir =
        fs::read_dir(&*IMAGE_DIR).expect("Failed to load images in fix_file_name");
    for photo in photos {
        let p: crate::fs::DirEntry = photo?;
        let file_name: OsString = p.file_name();
        let file_name: std::borrow::Cow<'_, str> = file_name.to_string_lossy();

        // Obtain possible old nickname
        if let Some(m) = re.captures(&file_name).and_then(|c| c.get(1))
            && !m.as_str().eq_ignore_ascii_case(nickname) {

            // Compare old and new nicknames case-insensitively
            let old_path = p.path();

            // Create a new file path with the updated nickname
            let new_path = Path::new(&*IMAGE_DIR).join(format!("takki_{}_{}.jpg", user_id, nickname));

            // Rename the file
            match fs::rename(&old_path, &new_path) {
                Ok(()) => println!("File renamed from {:?} to {:?}", old_path, new_path),
                Err(e) => println!("Failed to rename file: {}", e),
            }
        }
    }
    Ok(())
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "")]
pub enum Command {
    #[command(description = "Lähettää kuvan takistasi")]
    MunTakki,
    #[command(description = "Lähettää kuvan kaverin takista")]
    SunTakki,
    #[command(description = "Näyttää tämän viestin")]
    Help,
    #[command(description = "Näyttää tervetuliaisviestin")]
    Start,
}
