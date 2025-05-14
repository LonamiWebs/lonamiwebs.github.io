use std::path::PathBuf;
use std::{fs, io};

use crate::conf::INPUT_FOLDER;
use crate::{conf, date, markdown};

pub struct Entry {
    pub path: PathBuf,
    pub contents: Vec<u8>,
    pub permalink: String,
    pub title: String,
    pub date: String,
    pub updated: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
}

fn meta_string(value: &[u8]) -> String {
    String::from_utf8(value.to_vec()).expect("string to be valid utf8")
}

pub fn from_markdown(path: PathBuf, contents: Vec<u8>) -> Entry {
    let mut title = Option::<String>::None;
    let mut date = Option::<String>::None;
    let mut updated = Option::<String>::None;
    let mut category = Option::<String>::None;
    let mut tags = Vec::<String>::new();

    for token in markdown::parse(&contents).tokens {
        match token {
            markdown::Token::Meta(meta) => {
                title = meta.get(&b"title"[..]).map(|v| meta_string(v[0]));
                date = meta.get(&b"date"[..]).map(|v| meta_string(v[0]));
                updated = meta.get(&b"updated"[..]).map(|v| meta_string(v[0]));
                category = meta.get(&b"category"[..]).map(|v| meta_string(v[0]));
                tags = meta
                    .get(&b"tags"[..])
                    .unwrap_or(&Vec::new())
                    .iter()
                    .copied()
                    .map(meta_string)
                    .collect();

                if title.is_some() {
                    break;
                }
            }
            markdown::Token::Heading { level, text } => {
                if level == 1 {
                    title = Some(meta_string(text));
                }
                break;
            }
            _ => continue,
        }
    }

    let mut path_without_index = path.clone();
    if path_without_index.file_name().and_then(|s| s.to_str()) == Some("index.md") {
        path_without_index.pop();
    }

    let mut permalink = path_without_index
        .with_extension("")
        .to_str()
        .expect("path to be stringifiable")
        .replace('\\', "/");
    permalink.replace_range(..0, "/");

    let title = title.unwrap_or_else(|| {
        path_without_index
            .file_stem()
            .or_else(|| path.file_stem())
            .expect("file to have stem")
            .to_string_lossy()
            .to_string()
    });

    let mut input_path = PathBuf::from(conf::INPUT_FOLDER);
    input_path.push(&path);
    let date = date.unwrap_or_else(|| {
        date::system_time_to_date_string(
            fs::metadata(input_path)
                .expect("file metadata to be readable")
                .modified()
                .expect("file modified metadata to exist"),
        )
    });

    Entry {
        path,
        contents,
        permalink,
        title,
        date,
        updated,
        category,
        tags,
    }
}

pub fn from_markdown_in_path(folder: &str) -> Vec<Entry> {
    let mut path = PathBuf::from(INPUT_FOLDER);
    path.push(folder);

    fs::read_dir(path)
        .expect("folder path to be readable")
        .filter_map(|entry| {
            let entry = entry.expect("entry to be readable");
            let file_type = entry.file_type().expect("file type to be accessible");
            let mut path = entry.path();

            if file_type.is_dir() {
                path.push("index.md");
            } else if !file_type.is_file()
                || path.extension().and_then(|e| e.to_str()) != Some("md")
            {
                return None;
            }

            let contents = match fs::read(&path) {
                Ok(x) => x,
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    return None;
                }
                Err(_) => {
                    panic!("expected path to be readable");
                }
            };

            Some(from_markdown(
                path.strip_prefix(INPUT_FOLDER)
                    .expect("path to be inside input folder")
                    .to_owned(),
                contents,
            ))
        })
        .collect()
}
