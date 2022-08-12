use fltk::image::PngImage;

use crate::adventure::*;

use std::fs::{read_dir, File};
use std::io::Read;
use std::path::PathBuf;
use std::vec::Vec;

// expected paths where adventure data is stored
macro_rules! paths {
    ($path:expr) => {
        [
            ["$Home", ".local", "share", "adventure-book", $path].iter().collect::<PathBuf>(),
            ["usr", "share", "adventure-book", $path].iter().collect::<PathBuf>(),
            [".", "data", $path].iter().collect::<PathBuf>(),
        ]

    };
}

/// Iterates over folders with adventure data and collects all possible adventures to run
pub fn capture_adventures() -> Vec<Adventure> {
    let mut ret = Vec::<Adventure>::new();


    // going over the paths
    for path in paths!("books") {
        // reading all the directories on path
        if let Ok(it) = read_dir(path) {
            // going over directories, those are adventure folders
            for dir in it {
                // reading the folder data if it opens correctly
                if let Ok(dir) = dir {
                    // capturing the path to adventure metadata file
                    let mut path = dir.path();
                    path.push("adventure.txt");
                    // the file might not exist, if so we skip the folder
                    if path.exists() == false {
                        continue;
                    }

                    // we prepare a buffer for the file contents
                    let mut text = String::new();

                    // now we actually open the file on the path or skip the folder if it fails
                    let mut file = match File::open(path.clone()) {
                        Ok(t) => t,
                        _ => continue,
                    };
                    // and then we read the file, skipping the folder if it fails
                    if let Err(_) = file.read_to_string(&mut text) {
                        continue;
                    }

                    // removing the last bit to get folder path
                    path.pop();

                    // then we save it into our adventure for future reference
                    let path_text = match path.to_str() {
                        None => continue,
                        Some(p) => p.to_string(),
                    };
                    // next we parse the text into adventure. Parsing can fail if the text file isn't correctly formated or is incomplete, we skip over those.
                    let adventure = match Adventure::parse_from_string(text, path_text) {
                        Err(_) => continue,
                        Ok(a) => a,
                    };

                    // and at last we push the completed adventure metadata
                    ret.push(adventure);
                }
            }
        }
    }

    ret
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
