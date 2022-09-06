use fltk::image::PngImage;

use crate::adventure::*;

use std::fs::{read_dir, File};
use std::io::Read;
use std::path::PathBuf;
use std::vec::Vec;
pub(crate) use crate::dialog::signal_error;

pub enum FileError {
    LoadingFailure(PathBuf),
    ParsingFailure(PathBuf),
    FileUnopenable(PathBuf),
    CannotStringifyPathBuff(PathBuf),
    NoAdventureOnPath(PathBuf),
}
// Expected paths where adventure data is stored
macro_rules! paths {
    ($path:expr) => {
        [
            ["$Home", ".local", "share", "adventure-book", $path]
                .iter()
                .collect::<PathBuf>(),
            [".", "data", $path].iter().collect::<PathBuf>(),
            ["usr", "share", "adventure-book", $path]
                .iter()
                .collect::<PathBuf>(),
        ]
    };
}

/// Iterates over folders with adventure data and collects all possible adventures to run
pub fn capture_adventures() -> Vec<Adventure> {
    let mut ret = Vec::<Adventure>::new();

    // TODO support adventures in nested folders
    // going over the paths
    for path in paths!("books") {
        // reading all the directories on path
        if let Ok(it) = read_dir(path) {
            // going over directories, those are adventure folders
            for dir in it {
                // reading the folder data if it opens correctly
                if let Ok(dir) = dir {
                    // capturing the path to adventure metadata file
                    let path = dir.path();
                    match load_adventure(path) {
                        Err(e) => match e {
                            FileError::LoadingFailure(p) => signal_error!("Could not load file {}", p.to_str().unwrap()),
                            FileError::ParsingFailure(p) => signal_error!("Could not parse file {}", p.to_str().unwrap()),
                            FileError::FileUnopenable(p) => signal_error!("Could not open file {}", p.to_str().unwrap()),
                            FileError::CannotStringifyPathBuff(p) => signal_error!("Could not process path {:?}", p),
                            FileError::NoAdventureOnPath(p) => signal_error!("Could not find adventure on path {}", p.to_str().unwrap()),
                        }
                        Ok(adventure) => ret.push(adventure),
                    }
                }
            }
        }
    }

    ret
}
/// Loads adventure from provided path or returns nothing if path doesn't contain an adventure
pub fn load_adventure(path: PathBuf) -> Result<Adventure, FileError> {
    let mut path = path;
    // Saving off the path to adventure
    let path_text = match path.to_str() {
        None => return Err(FileError::CannotStringifyPathBuff(path)),
        Some(p) => p.to_string(),
    };

    path.push("adventure.txt");
    if is_adventure_on_path(&path) == false {
        return Err(FileError::NoAdventureOnPath(path));
    }
    // we prepare a buffer for the file contents
    let mut text = String::new();

    // now we actually open the file on the path or skip the folder if it fails
    let mut file = match File::open(path.clone()) {
        Ok(t) => t,
        _ => return Err(FileError::FileUnopenable(path)),
    };
    // and then we read the file, skipping the folder if it fails
    if let Err(_) = file.read_to_string(&mut text) {
        return Err(FileError::LoadingFailure(path));
    }

    // next we parse the text into adventure. Parsing can fail if the text file isn't correctly formated or is incomplete, we skip over those.
    match Adventure::parse_from_string(text, path_text) {
        Err(_) => Err(FileError::ParsingFailure(path)),
        Ok(a) => Ok(a),
    }
}
/// Tests if a path contains adventure files
pub fn is_adventure_on_path(path: &PathBuf) -> bool {
    let mut p = path.clone();
    if p.ends_with("adventure.txt") == false {
        p.push("adventure.txt");
    }
    p.exists()
}
/// Tests if the path is within a path from adventures can be read
pub fn is_on_adventure_path(path: &PathBuf) -> bool {
    for p in paths!("books") {
        if path.starts_with(p) {
            return true;
        }
    }
    false
}
/// Captures all pages from a path
///
/// path: Needs to be a valid path to a folder in which the adventure is stored, otherwise returned vec will be empty
pub fn capture_pages(path: &str) -> Vec<String> {
    let mut res = Vec::new();
    if let Ok(dir) = read_dir(path) {
        for file in dir {
            let file = match file {
                Ok(f) => f,
                Err(_) => continue,
            };

            let mut path = file.path();
            if path.is_dir() {
                continue;
            }
            match path.extension() {
                Some(ext) if ext != "txt" => continue,
                _ => {}
            }
            if path.ends_with("adventure.txt") {
                continue;
            }
            path.set_extension("");
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            res.push(name);
        }
    }
    res
}
/// Opens a page file and reads its contents, creating Page object
///
/// path: this is a path to adventure folder
/// name: this is a name of the page
///
/// The function automatically applies expected extension to the page name
pub fn read_page(path: &String, name: &String) -> Result<Page, String> {
    let mut path_to_file = PathBuf::new();
    path_to_file.push(path);
    path_to_file.push(name);
    path_to_file.set_extension("txt");

    if path_to_file.exists() == false {
        return Err(format!("Page '{}' doesn't exist", name));
    }

    // opening the file
    let mut p = match File::open(path_to_file.as_path()) {
        Err(_) => return Err(format!("Failed to open page {}", name)),
        Ok(f) => f,
    };

    // reading file's contents, in case it fails then we return error
    let mut text = String::new();
    if let Err(_) = p.read_to_string(&mut text) {
        return Err(format!("Failed to read page {}", name));
    }

    match Page::parse_from_string(text) {
        Err(e) => return Err(e),
        Ok(p) => return Ok(p),
    }
}
/// Loads image
///
/// name: file name
///
/// Function scans all known data paths in search of the image, supports png images only
pub fn get_image_png(name: &str) -> Result<PngImage, String> {
    for mut path in paths!("images") {
        path.push(name);
        if path.exists() {
            match PngImage::load(path) {
                Ok(v) => return Ok(v),
                Err(e) => return Err(format!("Couldn't load {}, {}", name, e)),
            }
        }
    }
    Err(format!("File {} not found", name))
}
