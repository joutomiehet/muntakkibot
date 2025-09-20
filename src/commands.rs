use crate::helpers::fix_file_name;
use regex::Regex;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use teloxide::{Bot, RequestError};

use crate::helpers::get_takki;

use crate::helpers::IMAGE_DIR;

pub async fn mun_takki(bot: &Bot, msg: &Message) -> Result<(), RequestError> {
    if let Some(user) = &msg.from {
        if let Some(nickname) = &user.username {
            let user_id = user.id.to_string();
            let _ = fix_file_name(&user_id, nickname, &*IMAGE_DIR);

            // Create regex to find takki by either ID or telegram nickname
            // Try id first in case nickname has changed
            let re: Regex =
                match Regex::new(&format!(r"(?i)takki_({}|{})_.*\.jpg", user_id, nickname)) {
                    Ok(r) => r,
                    Err(_) => {
                        bot.send_message(msg.chat.id, "Error in Takki regex")
                            .await?;
                        return Ok(());
                    }
                };

            let _ = get_takki(&msg, &bot, re, &nickname).await;
        }
    }
    Ok(())
}

pub async fn sun_takki(bot: &Bot, msg: &Message) -> Result<(), RequestError> {
    let message: Vec<&str> = match msg.text() {
        Some(text) => text.trim().split_whitespace().collect(),
        None => Vec::new(),
    };

    if message.len() < 2 {
        bot.send_message(msg.chat.id, "Umm unohditko laittaa nickin?")
            .await?;
    } else {
        for nick in &message[1..] {
            let nick_cleaned: &str = nick.trim_start_matches("@");
            let _ = match Regex::new(&format!(r"(?i)takki_.*_({}).jpg", nick_cleaned)) {
                Ok(re) => get_takki(&msg, &bot, re, nick_cleaned).await,
                Err(_) => {
                    println!("Regex error for nickname: {}", nick_cleaned);
                    Ok(())
                }
            };
        }
    }
    Ok(())
}

pub async fn help(bot: &Bot, msg: &Message) -> Result<(), RequestError> {
    let general_help: &str = "Tällä botilla voit tallentaa kuvan omasta Joutotakistasi ja näyttää siitä helposti kuvan kun tarvitset jonkun hakemaan takkisi solusta.\nRiittää että lähetät kuvan takistasi yksityisviestillä niin se tallentuu. Voit päivittää kuvan lähettämällä uuden kuvan.";
    let komennot: &str = "Komennot:\n/muntakki -> Lähettää kuvan sinun takistasi\n/suntakki @<tg-nicki> -> Hae toisen käyttäjän takki. Voit myös laitta monta käyttäjää peräkkäin samaan komentoon.";
    bot.send_message(msg.chat.id, format!("{}\n\n{}", general_help, komennot))
        .await?;
    Ok(())
}

pub async fn start(bot: &Bot, msg: &Message) -> Result<(), RequestError> {
    let start: &str = "Tervetuloa käyttämään MunTakkiBotia. Botti toimii hyvin yksinkertaisesti.\n1. Lähetä kuva takistasi minulle yksityisviestillä\n2. Käytä /muntakki komentoa chatissa.\n3. Käytä /suntakki <tg-nick> katsoaksesi jonkun muun takin.";
    bot.send_message(msg.chat.id, start).await?;
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
