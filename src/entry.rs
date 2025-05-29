use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::{conf, css, date, html, markdown, toml};

pub struct Entry {
    pub path: PathBuf,
    pub processed_path: PathBuf,
    pub processed_contents: Vec<u8>,
    pub append_css_style: Vec<u8>,
    pub append_siblings_listing: bool,
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

fn meta_date(value: &[u8]) -> String {
    const YMD_FMT: &[u8] = b"YYYY-MM-DD";
    meta_string(&value[..YMD_FMT.len().min(value.len())])
}

fn from_markdown(path: PathBuf, contents: Vec<u8>) -> Entry {
    let mut entry = from_existing_path(path, contents);
    let mut next_is_title = false;

    for token in markdown::lex(&entry.processed_contents) {
        match token {
            markdown::Token::Meta(meta) => {
                let meta = toml::parse(meta);
                entry.date = meta
                    .get(&b"date"[..])
                    .map(|v| meta_date(v[0]))
                    .unwrap_or(entry.date);
                entry.updated = meta.get(&b"updated"[..]).map(|v| meta_date(v[0]));
                entry.category = meta.get(&b"category"[..]).map(|v| meta_string(v[0]));
                entry.tags = meta
                    .get(&b"tags"[..])
                    .unwrap_or(&Vec::new())
                    .iter()
                    .copied()
                    .map(meta_string)
                    .collect();

                if let Some(title) = meta.get(&b"title"[..]) {
                    entry.title = meta_string(title[0]);
                    break;
                }
            }
            markdown::Token::Heading(1) => {
                next_is_title = true;
            }
            markdown::Token::Text(text) if next_is_title => {
                entry.title = meta_string(text);
                break;
            }
            _ => continue,
        }
    }
    let parsed = markdown::parse(markdown::lex(&entry.processed_contents));
    entry.append_css_style = parsed.additional_style;
    entry.processed_contents = html::minify(&html::generate(parsed.ast));

    const INDEX_LISTING: &str = "/_index.md";
    const INDEX_NESTED: &str = "/index.md";
    const MD_EXT: &str = ".md";
    let truncate = if entry.permalink.ends_with(INDEX_LISTING) {
        entry.append_siblings_listing = true;
        entry.processed_path.set_file_name("index.html");
        INDEX_LISTING.len() - 1
    } else if entry.permalink.ends_with(INDEX_NESTED) {
        entry.processed_path.set_extension("html");
        INDEX_NESTED.len() - 1
    } else {
        entry.processed_path.set_extension("");
        entry.processed_path.push("index.html");
        MD_EXT.len()
    };
    entry.permalink.truncate(entry.permalink.len() - truncate);
    if !entry.permalink.ends_with('/') {
        entry.permalink.push('/');
    }

    entry
}

fn from_css(path: PathBuf, contents: Vec<u8>) -> Entry {
    let mut entry = from_existing_path(path, contents);
    entry.processed_contents = css::minify(&entry.processed_contents);
    entry
}

fn from_html(path: PathBuf, contents: Vec<u8>) -> Entry {
    let mut entry = from_existing_path(path, contents);
    entry.processed_contents = html::minify(&entry.processed_contents);
    const INDEX_NAME: &str = "/index.html";
    if entry.permalink.ends_with(INDEX_NAME) {
        entry
            .permalink
            .truncate(entry.permalink.len() - (INDEX_NAME.len() - 1));
    }
    entry
}

fn from_existing_path(path: PathBuf, contents: Vec<u8>) -> Entry {
    let mut entry = from_new_path(path, contents);
    entry.date = date::system_time_to_date_string(
        fs::metadata(&entry.path)
            .expect("file metadata to be readable")
            .modified()
            .expect("file modified metadata to exist"),
    );
    entry
}

fn from_new_path(path: PathBuf, contents: Vec<u8>) -> Entry {
    let title = match path.file_stem().expect("path to have file stem") {
        stem if stem == "index" => path
            .parent()
            .expect("path to have parent")
            .file_name()
            .expect("path to have file name"),
        stem => stem,
    }
    .to_string_lossy()
    .into_owned();

    let processed_path = path
        .strip_prefix(conf::INPUT_FOLDER)
        .unwrap_or(&path)
        .to_owned();

    let mut permalink = processed_path
        .to_str()
        .expect("path to be stringifiable")
        .replace('\\', "/");
    permalink.replace_range(..0, "/");

    Entry {
        path,
        processed_path,
        processed_contents: contents,
        append_css_style: Vec::new(),
        append_siblings_listing: false,
        permalink,
        title,
        date: String::new(),
        updated: None,
        category: None,
        tags: Vec::new(),
    }
}

impl Entry {
    pub fn load_from_path(path: PathBuf) -> io::Result<Self> {
        let contents = fs::read(&path)?;
        Ok(match path.extension().and_then(|e| e.to_str()) {
            Some("md") => from_markdown(path, contents),
            Some("css") => from_css(path, contents),
            Some("html") => from_html(path, contents),
            _ => from_existing_path(path, contents),
        })
    }

    pub fn from_new_path_with_contents(path: PathBuf, contents: Vec<u8>) -> Self {
        from_new_path(path, contents)
    }

    pub fn path_parent(&self) -> Option<&Path> {
        if self.path.file_stem().is_some_and(|s| s == "index") {
            self.path.parent().and_then(|p| p.parent())
        } else {
            self.path.parent()
        }
    }
}
