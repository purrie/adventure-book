use dirs::{cache_dir, data_dir};
use fltk::app;
use fltk::image::PngImage;

use crate::adventure::*;

pub(crate) use crate::dialog::signal_error;
use std::fmt::Display;
use std::fs::{create_dir_all, read_dir, remove_dir_all, remove_file, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::vec::Vec;

#[derive(Debug)]
pub enum FileError {
    LoadingFailure(PathBuf),
    ParsingFailure(PathBuf, ParsingError),
    FileUnopenable(PathBuf),
    CannotStringifyPathBuff(PathBuf),
    NoAdventureOnPath(PathBuf),
    FileNonExistent(PathBuf),
}
pub const PROJECT_PATH_NAME: &str = "adventure-book";
/// Expected paths where adventure data is stored for user created content on windows
#[cfg(target_os = "windows")]
macro_rules! user_paths {
    ($path:expr) => {
        [
            [
                data_dir().unwrap().to_str().unwrap(),
                PROJECT_PATH_NAME,
                $path,
            ]
            .iter()
            .collect::<PathBuf>(),
            [".", "data", $path].iter().collect::<PathBuf>(),
        ]
    };
}
/// Expected paths where adventure data is stored for user created content on linux
#[cfg(target_os = "linux")]
macro_rules! user_paths {
    ($path:expr) => {
        [
            [
                data_dir().unwrap().to_str().unwrap(),
                PROJECT_PATH_NAME,
                $path,
            ]
            .iter()
            .collect::<PathBuf>(),
        ]
    };
}
/// Expected paths where adventure and core program data is stored on windows
#[cfg(target_os = "windows")]
macro_rules! all_paths {
    ($path:expr) => {
        [
            [
                data_dir().unwrap().to_str().unwrap(),
                PROJECT_PATH_NAME,
                $path,
            ]
            .iter()
            .collect::<PathBuf>(),
            [".", "data", $path].iter().collect::<PathBuf>(),
        ]
    };
}
/// Expected paths where adventure and core program data is stored on dev builds
#[cfg(target_os = "linux")]
#[cfg(debug_assertions)]
macro_rules! all_paths {
    ($path:expr) => {
        [
            [
                data_dir().unwrap().to_str().unwrap(),
                PROJECT_PATH_NAME,
                $path,
            ]
            .iter()
            .collect::<PathBuf>(),
            [".", "data", $path].iter().collect::<PathBuf>(),
            ["/", "usr", "share", PROJECT_PATH_NAME, $path].iter().collect::<PathBuf>(),
        ]
    };
}
/// Expected paths where adventure and core program data is stored on Linux
#[cfg(target_os = "linux")]
#[cfg(not(debug_assertions))]
macro_rules! all_paths {
    ($path:expr) => {
        [
            [
                data_dir().unwrap().to_str().unwrap(),
                PROJECT_PATH_NAME,
                $path,
            ]
            .iter()
            .collect::<PathBuf>(),
            ["/", "usr", "share", PROJECT_PATH_NAME, $path].iter().collect::<PathBuf>(),
        ]
    };
}
pub(crate) use all_paths;
pub(crate) use user_paths;

impl Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::LoadingFailure(p) => {
                write!(f, "Could not load file {}", p.to_str().unwrap())
            }
            FileError::ParsingFailure(p, e) => write!(
                f,
                "Could not parse file {} because of {}",
                p.to_str().unwrap(),
                e
            ),
            FileError::FileUnopenable(p) => {
                write!(f, "Could not open file {}", p.to_str().unwrap())
            }
            FileError::CannotStringifyPathBuff(p) => {
                write!(f, "Could not process path {:?}", p.to_str().unwrap())
            }
            FileError::NoAdventureOnPath(p) => write!(
                f,
                "Could not find adventure on path {}",
                p.to_str().unwrap()
            ),
            FileError::FileNonExistent(p) => {
                write!(f, "File doesn't exist: {}", p.to_str().unwrap())
            }
        }
    }
}
/// Iterates over folders with adventure data and collects all possible adventures to run
pub fn capture_adventures() -> Vec<Adventure> {
    let mut ret = Vec::<Adventure>::new();

    // going over the paths
    for path in all_paths!("books") {
        // reading all the directories on path
        if let Ok(it) = read_dir(path) {
            // going over directories, those are adventure folders
            for dir in it {
                // reading the folder data if it opens correctly
                if let Ok(dir) = dir {
                    // capturing the path to adventure metadata file
                    let path = dir.path();
                    match load_adventure(path) {
                        Err(e) => signal_error!("{}", e),
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
        Err(e) => Err(FileError::ParsingFailure(path, e)),
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
    let expected_paths = user_paths!("books").map(|x| {
        if x.is_absolute() {
            return x;
        } else {
            if let Ok(p) = x.canonicalize() {
                return p;
            } else {
                unreachable!()
            }
        }
    });
    if path.is_absolute() {
        if expected_paths.iter().any(|x| path.starts_with(x)) {
            return true;
        }
    } else {
        if let Ok(p) = path.canonicalize() {
            if expected_paths.iter().any(|x| p.starts_with(x)) {
                return true;
            }
        }
    }
    false
}
/// Removes a folder with the adventure
pub fn remove_adventure<P: AsRef<Path>>(path: P) {
    // TODO this should deliberately delete just the text files to avoid removing things unrelated to the adventure
    match remove_dir_all(path) {
        Ok(_) => {}
        Err(_) => {}
    }
}
/// Writes adventure metadata into file
///
/// path: adventure path, should be the same as stored in adventure struct
/// serialized_adventure: result of calling serialize_to_string on an adventure struct
pub fn save_adventure(path: &str, serialized_adventure: String) {
    let mut path = PathBuf::from(path);
    if path.exists() == false {
        match create_dir_all(&path) {
            Ok(_) => {}
            Err(_) => {
                println!("Path {:?} could not be created!", path.to_str());
                return;
            }
        }
    }
    path.push("adventure");
    path.set_extension("txt");
    if let Ok(mut file) = File::create(path) {
        if let Err(e) = file.write(serialized_adventure.as_bytes()) {
            signal_error!("Error saving the adventure metadata: {}", e);
        }
    }
}
/// Writes a page into file
///
/// path: adventure path, should be the same as stored in adventure struct
/// file_name: name of the file, needs to be correct and the same as referred to by StoryResult structs in other pages that lead to this page
/// serialized_page: result of calling serialize_to_string on a page
pub fn save_page(path: &str, file_name: String, serialized_page: String) {
    let mut path = PathBuf::from(path);
    if path.exists() == false {
        match create_dir_all(&path) {
            Ok(_) => {}
            Err(_) => {
                println!("Path {:?} could not be created!", path.to_str());
                return;
            }
        }
    }
    path.push(&file_name);
    path.set_extension("txt");
    if let Ok(mut file) = File::create(path) {
        if let Err(e) = file.write(serialized_page.as_bytes()) {
            signal_error!("Error saving the page {}: {}", file_name, e);
        }
    }
}
/// Tests if the file name is valid
///
/// there's probably a better way to do it, but for now, it saves a temporary dummy file with the name to drive, if it succeeds, it is considered valid
pub fn is_valid_file_name(name: &str) -> bool {
    let mut test_path = cache_dir().unwrap();
    test_path.push(PROJECT_PATH_NAME);
    if test_path.exists() == false {
        create_dir_all(&test_path).unwrap();
    }
    test_path.push(name);
    test_path.set_extension("txt");
    if let Ok(_f) = File::create(&test_path) {
        remove_file(test_path).unwrap();
        return true;
    } else {
        return false;
    }
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
    res.sort();
    res
}
/// Opens a page file and reads its contents, creating Page object
///
/// path: this is a path to adventure folder
/// name: this is a name of the page
///
/// The function automatically applies expected extension to the page name
pub fn read_page(path: &String, name: &String) -> Result<Page, FileError> {
    let mut path_to_file = PathBuf::new();
    path_to_file.push(path);
    path_to_file.push(name);
    path_to_file.set_extension("txt");

    if path_to_file.exists() == false {
        return Err(FileError::FileNonExistent(path_to_file));
    }

    // opening the file
    let mut p = match File::open(path_to_file.as_path()) {
        Err(_) => return Err(FileError::FileUnopenable(path_to_file)),
        Ok(f) => f,
    };

    // reading file's contents, in case it fails then we return error
    let mut text = String::new();
    if let Err(_) = p.read_to_string(&mut text) {
        return Err(FileError::CannotStringifyPathBuff(path_to_file));
    }

    match Page::parse_from_string(text) {
        Err(e) => return Err(FileError::ParsingFailure(path_to_file, e)),
        Ok(p) => return Ok(p),
    }
}
/// Loads image
///
/// name: file name
///
/// Function scans all known data paths in search of the image, supports png images only
pub fn get_image_png(name: &str) -> Result<PngImage, String> {
    for mut path in all_paths!("images") {
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
/// Opens a help page by name
///
/// Only the name is necessary, the function will apply the extension and the path
pub fn open_help(name: &str) {
    for mut path in all_paths!("help") {
        path.push(name);
        path.set_extension("html");
        if path.exists() {
            let mut help = fltk::dialog::HelpDialog::new(100, 100, 800, 600);
            if let Err(e) = help.load(path) {
                signal_error!("Error opening a help page: {}", e);
            } else {
                help.show();
                while help.shown() {
                    app::wait();
                }
                return;
            }
        }
    }
    signal_error!("Could not find a help page: {}", name);
}
