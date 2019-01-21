//! Handles requests for `ascii-star` data.
//! 
//! This server can serve the `.txt` files containing the lyrics and `.mp3` files containing the
//! actual song.
//!
//! It also supports searching for songs.
#![feature(proc_macro_hygiene, decl_macro)]

mod configuration;

use rocket::{get, routes, response::NamedFile};
use rocket_contrib::{json, json::JsonValue};
use std::{io::Read, fs::{read_dir, File}, path::{PathBuf, Path}};

/// Returns an mp3 file at the specified path.
#[get("/mp3/<path..>")]
fn get_mp3(path: PathBuf) -> Option<NamedFile> {
    let mut file_path = PathBuf::from(crate::configuration::MP3_PATH);

    file_path.push(path);

    NamedFile::open(file_path).ok()
}

/// Returns a song `.txt` file at the specified path.
#[get("/song/<path..>")]
fn get_song_txt(path: PathBuf) -> Option<NamedFile> {
    let mut file_path = PathBuf::from(crate::configuration::SONG_PATH);

    file_path.push(path);

    NamedFile::open(file_path).ok()
}

/// Returns all the files that match the search.
#[get("/search?<q>")]
fn search(q: String) -> Option<JsonValue> {
    let mut results = Vec::new();
    let search = q.to_lowercase();
    let search_words: Vec<_> = search
        .split_whitespace()
        .map(|word| word.as_ref())
        .collect();

    for entry in read_dir(crate::configuration::SONG_PATH).ok()? {
        if let Ok(entry) = entry {
            if let Some(header) = get_matching_header(&entry.path(), &search_words) {
                if let Ok(name) = entry.file_name().into_string() {
                    let path: PathBuf = ["song", &name].iter().collect();
                    results.push(
                        json!({
                            "path": path,
                            "title": header.title,
                            "artist": header.artist,
                            "genre": header.genre
                        })
                    );
                }
            }
        }
    }

    Some(json!({"results": results}))
}

/// Determines if the given file matches the search and returns the header if it does.
fn get_matching_header(path: &Path, search_words: &[&str]) -> Option<ultrastar_txt::Header> {
    let mut content = String::new();
    let mut file = File::open(path).ok()?;

    file.read_to_string(&mut content).ok()?;

    let original_header = ultrastar_txt::parse_txt_header_str(&content).ok()?;

    let mut header = original_header.clone();

    // Make the search case insensitive
    header.artist = header.artist.to_lowercase();
    header.title = header.title.to_lowercase();
    header.genre = header.genre.map(|genre| genre.to_lowercase());

    if search_words
        .iter()
        .all(|word|
            matches_header(&header, &*word)
        ) {
        Some(original_header)
    } else {
        None
    }
}

/// Determines if the given ultastar header matches the search word.
fn matches_header(header: &ultrastar_txt::Header, word: &str) -> bool {
    header.artist.contains(word)
        || header.title.contains(word)
        || header.genre.as_ref().map(|genre| genre.contains(word)).unwrap_or(false)
}

/// Starts the server.
fn main() {
    rocket::ignite()
        .mount("/",
               routes![
                   get_mp3,
                   get_song_txt,
                   search
               ]
        ).launch();
}
