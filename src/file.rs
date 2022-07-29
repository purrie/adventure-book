use crate::adventure::*;

use std::fs::{read_dir, File};
use std::io::Read;
use std::path::PathBuf;
use std::vec::Vec;

/// Iterates over folders with adventure data and collects all possible adventures to run
pub fn capture_adventures() -> Vec<Adventure> {
    let mut ret = Vec::<Adventure>::new();

    // expected paths where adventure data is stored
    let paths = [
        "$HOME/.local/share/adventure-book/",
        "/usr/share/adventure-book/",
        "./books/",
    ];

    // going over the paths
    for path in paths {
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
pub fn read_page(path: &String, name: &String) -> Result<Page, ()> {
    let mut path_to_file = PathBuf::new();
    path_to_file.push(path);
    path_to_file.push(name);
    path_to_file.set_extension("txt");

    if path_to_file.exists() == false {
        return Err(());
    }

    // opening the file
    let mut p = match File::open(path_to_file.as_path()) {
        Err(_) => return Err(()),
        Ok(f) => f,
    };

    // reading file's contents, in case it fails then we return error
    let mut text = String::new();
    if let Err(_) = p.read_to_string(&mut text) {
        return Err(());
    }

    match Page::parse_from_string(text) {
        Err(_) => return Err(()),
        Ok(p) => return Ok(p),
    }
}

