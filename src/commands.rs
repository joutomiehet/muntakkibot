use crate::helpers::{IMAGE_DIR, find_by_nick, fix_file_name, get_takki};
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use teloxide::{Bot, RequestError};

pub async fn mun_takki(bot: &Bot, msg: &Message) -> Result<(), RequestError> {
    if let Some(user) = &msg.from {
        let nickname: &str = user.username.as_deref().unwrap_or_default();
        let name: String = match &user.username {
            Some(nickname) => format!("@{}", nickname),
            None => user.first_name.clone(),
        };

        // Find takki by ID, the nickname might have changed
        let photo = fix_file_name(&user.id.to_string(), nickname, &IMAGE_DIR)?;
        get_takki(msg, bot, photo, &name).await?;
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
            let photo = find_by_nick(&IMAGE_DIR, nick_cleaned)?;
            get_takki(msg, bot, photo, &format!("@{}", nick_cleaned)).await?;
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
