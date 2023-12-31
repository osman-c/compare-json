use clap::Parser;
use serde_json;
use std::collections::{HashMap, BTreeMap};
use std::fs::{DirEntry, File};
use std::io::BufReader;
use std::{env, fs};

#[derive(Debug)]
struct NameSpace {
    name: String,
    data: HashMap<String, String>,
}

#[derive(Debug)]
struct Language {
    name: String,
    files: Vec<NameSpace>,
}

#[derive(Debug)]
struct GlobalNameSpace {
    name: String,
    keys: Vec<String>,
}

#[derive(Debug)]
struct Stack {
    values: Vec<GlobalNameSpace>,
}

/// Simple program to check your locales files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command()]
    directory: String,
    /// Also sort locale files
    #[arg(long, short, action)]
    sort: bool,
}

impl Stack {
    fn add_name_space(&mut self, name: String, keys: Vec<String>) -> () {
        self.values.push(GlobalNameSpace { name, keys });
    }

    fn merge_keys(&mut self, name: String, keys: Vec<String>) -> () {
        let edited = self.values.iter().position(|n| n.name == name);

        if let Some(found) = edited {
            self.values[found].keys.extend(keys);
            self.values[found].keys.sort();
            self.values[found].keys.dedup();
        } else {
            self.add_name_space(name.clone(), keys)
        }
    }
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let args = Args::parse();
    print!("{}", args.directory);
    let paths = fs::read_dir(&args.directory).expect("Not a directory");
    let mut stack = Stack { values: Vec::new() };
    let languages: Vec<Language> = paths
        .into_iter()
        .filter_map(|l| -> Option<Language> { break_into_lang(l, &mut stack, &args) })
        .collect();

    for global_name_space in stack.values {
        println!("Looking at namespace '{}'", global_name_space.name);
        for lan in &languages {
            let language_name_space = lan.files.iter().find(|n| n.name == global_name_space.name);
            if let Some(found) = language_name_space {
                check(found, &global_name_space, &lan.name)
            }
        }
    }
}

fn break_into_lang(
    folder: Result<DirEntry, std::io::Error>,
    stack: &mut Stack,
    args: &Args,
) -> Option<Language> {
    let _folder = folder.ok()?;
    let name = get_folder_name(&_folder);
    let files = _folder
        .path()
        .read_dir()
        .ok()?
        .into_iter()
        .filter_map(|l| -> Option<NameSpace> { break_into_hash(l, stack, args) })
        .collect();

    Some(Language { name, files })
}

fn break_into_hash(
    folder: Result<DirEntry, std::io::Error>,
    stack: &mut Stack,
    args: &Args,
) -> Option<NameSpace> {
    let folder = folder.ok()?;
    let name = get_folder_name(&folder);
    let file = File::open(folder.path()).ok()?;
    let reader = BufReader::new(&file);

    let data: HashMap<String, String> = serde_json::from_reader(reader).ok()?;
    let keys: Vec<String> = data.keys().cloned().collect();
    stack.merge_keys(name.clone(), keys);

    if args.sort {
        println!("Sorting {}", folder.path().display());
        let sorted_vector: Vec<(String, String)> = data.clone().into_iter().collect();
        let sorted_btree_map: BTreeMap<String, String> = sorted_vector.into_iter().collect();

        let pretty_json_result = serde_json::to_string_pretty(&sorted_btree_map);
        let pretty_json = match pretty_json_result {
            Ok(file) => file,
            Err(error) => {
                dbg!(error);
                return None;
            }
        };

        fs::write(folder.path(), pretty_json).unwrap();
    }

    Some(NameSpace { name, data })
}

fn get_folder_name(folder: &DirEntry) -> String {
    folder
        .path()
        .display()
        .to_string()
        .split("/")
        .last()
        .unwrap_or("unknown folder")
        .to_string()
}

fn check(ns: &NameSpace, g: &GlobalNameSpace, locale: &String) {
    for k in &g.keys {
        let value = ns.data.get(k);
        if let None = value {
            println!("Key '{}' is missing in '{}' locale", k, locale);
        }
    }
}
