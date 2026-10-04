use once_cell::sync::Lazy;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use teloxide::{RequestError, prelude::*, types::*};
pub static IMAGE_DIR: Lazy<String> = Lazy::new(|| env::var("IMAGE_DIR").unwrap_or(".".to_string()));

// Photos are named takki_<id>_<nickname>.jpg (or takki_<id>.jpg in the old format).
// The user id is what identifies the owner, the nickname is only there for /suntakki.
fn parse_file_name(file_name: &str) -> Option<(&str, &str)> {
    let rest = file_name.strip_prefix("takki_")?.strip_suffix(".jpg")?;
    let (id, nickname) = rest.split_once('_').unwrap_or((rest, ""));
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some((id, nickname))
}

fn takki_path(dir: &str, user_id: &str, nickname: &str) -> PathBuf {
    Path::new(dir).join(format!("takki_{}_{}.jpg", user_id, nickname))
}

fn find_takkis(dir: &str, matches: impl Fn(&str, &str) -> bool) -> io::Result<Vec<PathBuf>> {
    Ok(fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|entry| {
            parse_file_name(&entry.file_name().to_string_lossy())
                .is_some_and(|(id, nickname)| matches(id, nickname))
        })
        .map(|entry| entry.path())
        .collect())
}

fn modified(path: &Path) -> SystemTime {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

pub fn find_by_id(dir: &str, user_id: &str) -> io::Result<Vec<PathBuf>> {
    find_takkis(dir, |id, _| id == user_id)
}

pub fn find_by_nick(dir: &str, nickname: &str) -> io::Result<Option<PathBuf>> {
    if nickname.is_empty() {
        return Ok(None);
    }
    let found = find_takkis(dir, |_, nick| nick.eq_ignore_ascii_case(nickname))?;
    Ok(found.into_iter().max_by_key(|p| modified(p)))
}

/// Saves a new photo for the user and removes all their older photos
pub fn save_takki(dir: &str, user_id: &str, nickname: &str, bytes: &[u8]) -> io::Result<()> {
    let path = takki_path(dir, user_id, nickname);
    // Write to a temp file first so a half-written photo is never sent
    let tmp_path = path.with_extension("jpg.tmp");
    fs::write(&tmp_path, bytes)?;
    fs::rename(&tmp_path, &path)?;

    for old_path in find_by_id(dir, user_id)? {
        if old_path != path {
            if let Err(e) = fs::remove_file(&old_path) {
                println!("Oops couldn't remove old file {:?}: {}", old_path, e);
            }
        }
    }
    Ok(())
}

/// Makes sure the user has exactly one photo named with their current nickname.
/// Returns the path to the photo, if the user has one.
pub fn fix_file_name(user_id: &str, nickname: &str, dir: &str) -> io::Result<Option<PathBuf>> {
    let path = takki_path(dir, user_id, nickname);
    let photos = find_by_id(dir, user_id)?;

    // Keep the newest photo. If there's a tie, prefer the one already correctly named
    let Some(newest) = photos
        .iter()
        .max_by_key(|p| (modified(p), **p == path))
        .cloned()
    else {
        return Ok(None);
    };

    for photo in &photos {
        if *photo != newest {
            if let Err(e) = fs::remove_file(photo) {
                println!("Oops couldn't remove old file {:?}: {}", photo, e);
            }
        }
    }

    // Nickname has changed or the file is in the old format
    if newest != path {
        fs::rename(&newest, &path)?;
        println!("File renamed from {:?} to {:?}", newest, path);
    }
    Ok(Some(path))
}

pub async fn get_takki(
    msg: &Message,
    bot: &Bot,
    photo: Option<PathBuf>,
    name: &str,
) -> Result<(), RequestError> {
    if let Some(file_path) = photo {
        let file = InputFile::file(file_path);
        bot.send_photo(msg.chat.id, file)
            .caption(name.to_string())
            .await?;
    } else {
        bot.send_message(msg.chat.id, format!("Takkiasi ei löytynyt {}", name))
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::time::Duration;
    use tempfile::tempdir;

    fn set_age(path: &Path, seconds_ago: u64) {
        let time = SystemTime::now() - Duration::from_secs(seconds_ago);
        File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(time)
            .unwrap();
    }

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
        fs::write(&old_file, "old").unwrap();
        fs::write(&new_file, "new").unwrap();
        set_age(&old_file, 60);

        // Run the function
        fix_file_name(user_id, "nick", dir.path().to_str().unwrap()).unwrap();

        // Old file should be deleted, new one should still exist
        assert!(!old_file.exists());
        assert_eq!(fs::read_to_string(&new_file).unwrap(), "new");
    }

    #[test]
    fn test_upload_after_nick_change_keeps_new_photo() {
        let dir = tempdir().unwrap();
        let dir_str = dir.path().to_str().unwrap();
        let old_file = dir.path().join("takki_123_bob.jpg");
        let new_file = dir.path().join("takki_123_rob.jpg");
        fs::write(&old_file, "old").unwrap();

        save_takki(dir_str, "123", "rob", b"new").unwrap();

        assert!(!old_file.exists());
        assert_eq!(fs::read_to_string(&new_file).unwrap(), "new");
        assert_eq!(fix_file_name("123", "rob", dir_str).unwrap(), Some(new_file.clone()));
        assert_eq!(fs::read_to_string(&new_file).unwrap(), "new");
    }

    #[test]
    fn test_case_only_nick_change_leaves_one_file() {
        let dir = tempdir().unwrap();
        let dir_str = dir.path().to_str().unwrap();
        fs::write(dir.path().join("takki_123_Bob.jpg"), "old").unwrap();

        save_takki(dir_str, "123", "bob", b"new").unwrap();

        let photos = find_by_id(dir_str, "123").unwrap();
        assert_eq!(photos, vec![dir.path().join("takki_123_bob.jpg")]);
        assert_eq!(fs::read_to_string(&photos[0]).unwrap(), "new");
    }

    #[test]
    fn test_find_by_nick_is_exact() {
        let dir = tempdir().unwrap();
        let dir_str = dir.path().to_str().unwrap();
        File::create(dir.path().join("takki_1_a_bob.jpg")).unwrap();
        File::create(dir.path().join("takki_2_Bob.jpg")).unwrap();

        assert_eq!(
            find_by_nick(dir_str, "bob").unwrap(),
            Some(dir.path().join("takki_2_Bob.jpg"))
        );
        assert_eq!(
            find_by_nick(dir_str, "a_bob").unwrap(),
            Some(dir.path().join("takki_1_a_bob.jpg"))
        );
        assert_eq!(find_by_nick(dir_str, ".*").unwrap(), None);
        assert_eq!(find_by_nick(dir_str, "").unwrap(), None);
    }

    #[test]
    fn test_nick_change_with_underscore() {
        let dir = tempdir().unwrap();
        let dir_str = dir.path().to_str().unwrap();
        fs::write(dir.path().join("takki_123_Slim_Boii.jpg"), "photo").unwrap();

        let path = fix_file_name("123", "SlimBoii", dir_str).unwrap();

        assert_eq!(path, Some(dir.path().join("takki_123_SlimBoii.jpg")));
        assert_eq!(find_by_id(dir_str, "123").unwrap().len(), 1);
    }

    #[test]
    fn test_user_without_nickname() {
        let dir = tempdir().unwrap();
        let dir_str = dir.path().to_str().unwrap();

        save_takki(dir_str, "123", "", b"photo").unwrap();

        assert_eq!(
            fix_file_name("123", "", dir_str).unwrap(),
            Some(dir.path().join("takki_123_.jpg"))
        );
    }

    #[test]
    fn test_find_by_id_does_not_match_other_ids() {
        let dir = tempdir().unwrap();
        let dir_str = dir.path().to_str().unwrap();
        File::create(dir.path().join("takki_1234_nick.jpg")).unwrap();
        File::create(dir.path().join("takki_123_nick.jpg.tmp")).unwrap();

        assert!(find_by_id(dir_str, "123").unwrap().is_empty());
        assert_eq!(fix_file_name("123", "nick", dir_str).unwrap(), None);
    }
}
