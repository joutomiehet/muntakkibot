use once_cell::sync::Lazy;
use regex::Regex;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use teloxide::{RequestError, prelude::*, types::*};
pub static IMAGE_DIR: Lazy<String> = Lazy::new(|| env::var("IMAGE_DIR").unwrap_or("".to_string()));

pub fn fix_file_name(user_id: &str, nickname: &str, dir: &str) -> Result<(), RequestError> {
    // Check if there's possibly duplicate pictures
    let old_path = Path::new(dir).join(format!("takki_{}.jpg", user_id));
    let new_path = Path::new(dir).join(format!("takki_{}_{}.jpg", user_id, nickname));

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
            return Ok(());
        }
    };
    let photos: crate::fs::ReadDir =
        fs::read_dir(dir).expect("Failed to load images in fix_file_name");
    for photo in photos {
        let p: crate::fs::DirEntry = photo?;
        let file_name: OsString = p.file_name();
        let file_name: std::borrow::Cow<'_, str> = file_name.to_string_lossy();

        // Obtain possible old nickname
        if let Some(m) = re.captures(&file_name).and_then(|c| c.get(1))
            && !m.as_str().eq_ignore_ascii_case(nickname)
        {
            // Compare old and new nicknames case-insensitively
            let old_path = p.path();

            // Create a new file path with the updated nickname
            let new_path = Path::new(dir).join(format!("takki_{}_{}.jpg", user_id, nickname));

            // Rename the file
            match fs::rename(&old_path, &new_path) {
                Ok(()) => println!("File renamed from {:?} to {:?}", old_path, new_path),
                Err(e) => println!("Failed to rename file: {}", e),
            }
        }
    }
    Ok(())
}

pub async fn get_takki(
    msg: &Message,
    bot: &Bot,
    re: Regex,
    name: &str,
) -> Result<(), RequestError> {
    // We need to loop through all the files
    let found_photo = fs::read_dir(&*IMAGE_DIR)?
        .filter_map(Result::ok)
        .find(|entry| re.is_match(&entry.file_name().to_string_lossy()))
        .map(|e| e.path());

    if let Some(file_path) = found_photo {
        let file = InputFile::file(file_path);
        bot.send_photo(msg.chat.id, file)
            .caption(format!("@{}", name))
            .await?;
    } else {
        bot.send_message(msg.chat.id, format!("Takkiasi ei löytynyt @{}", name))
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_rename_old_file_to_new_nickname() {
        let dir = tempdir().unwrap();
        let user_id = "123";
        let old_file = dir.path().join(format!("takki_{}.jpg", user_id));
        let new_file = dir.path().join(format!("takki_{}_newnick.jpg", user_id));

        File::create(&old_file).unwrap();
        fix_file_name(user_id, "newnick", dir.path().to_str().unwrap()).unwrap();

        assert!(!old_file.exists());
        assert!(new_file.exists());
    }

    #[test]
    fn test_remove_duplicate_old_file() {
        let dir = tempdir().unwrap();
        let user_id = "123";
        let old_file = dir.path().join(format!("takki_{}.jpg", user_id));
        let new_file = dir.path().join(format!("takki_{}_nick.jpg", user_id));

        // Create both files
        File::create(&old_file).unwrap();
        File::create(&new_file).unwrap();

        // Run the function
        fix_file_name(user_id, "nick", dir.path().to_str().unwrap()).unwrap();

        // Old file should be deleted, new one should still exist
        assert!(!old_file.exists());
        assert!(new_file.exists());
    }
}
