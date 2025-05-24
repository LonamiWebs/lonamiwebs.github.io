use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process;
use std::{fs, thread};

pub mod cli;
pub mod collections;
pub mod conf;
pub mod css;
pub mod date;
pub mod entry;
pub mod file_watcher;
pub mod html;
pub mod http;
pub mod markdown;
pub mod template;
pub mod toml;
pub mod walkdir;
#[cfg(target_os = "windows")]
pub mod winapi;

fn load_template() -> Vec<u8> {
    let mut path = PathBuf::from(conf::INPUT_FOLDER);
    path.push(conf::TEMPLATE_NAME);
    html::minify(&fs::read(&path).expect("path to be a readable file"))
}

fn process_file(template: &[u8], path: &Path) -> Result<(PathBuf, Vec<u8>), Box<dyn Error>> {
    let contents = fs::read(path)?;
    let path = path.strip_prefix(conf::INPUT_FOLDER)?;

    let (path, contents) = match path.extension().and_then(|e| e.to_str()) {
        Some("md") => {
            let mut entry = entry::from_markdown(path.to_owned(), contents);
            entry.contents = html::minify(&html::generate(markdown::parse(markdown::lex(
                &entry.contents,
            ))));
            let contents = template::apply(template, entry);
            let path = if path.file_stem().and_then(|s| s.to_str()) == Some("index") {
                path.with_extension("html")
            } else {
                let mut path = path.with_extension("");
                path.push("index.html");
                path
            };
            (path, contents)
        }
        Some("css") => (path.to_owned(), css::minify(&contents)),
        Some("html") => (path.to_owned(), html::minify(&contents)),
        _ => (path.to_owned(), contents),
    };

    let mut output_path = PathBuf::from(conf::OUTPUT_FOLDER);
    output_path.push(path);
    Ok((output_path, contents))
}

fn commit_file(path: &Path, contents: &[u8]) {
    fs::create_dir_all(path.parent().expect("path to have a parent"))
        .expect("parent directories to be created");

    fs::write(path, contents).expect("path to be writable");
}

fn build(config: cli::BuildConfig) {
    let mut generated = HashMap::<PathBuf, Vec<u8>>::new();

    let mut cname_path = PathBuf::from(conf::OUTPUT_FOLDER);
    cname_path.push("CNAME");
    generated.insert(cname_path, conf::CNAME.as_bytes().to_vec());

    let template = load_template();
    for entry in walkdir::walk(PathBuf::from(conf::INPUT_FOLDER)) {
        if entry.file_name() == conf::TEMPLATE_NAME {
            continue;
        }

        let path = entry.path();
        match process_file(&template, &path) {
            Ok((path, contents)) => {
                generated.insert(path, contents);
            }
            Err(error) => {
                println!("failed to process file: {path:?}\n  {error}");
                if config.ignore_errors {
                    continue;
                }
                process::exit(1);
            }
        }
    }

    if config.write {
        if config.force {
            let _ = fs::remove_dir_all(conf::OUTPUT_FOLDER);
        }

        for (path, contents) in generated {
            commit_file(&path, &contents);
        }
    }
}

fn serve(config: cli::ServeConfig) {
    if config.watch {
        thread::spawn(|| {
            let template = load_template();
            for path in file_watcher::watch(conf::INPUT_FOLDER) {
                if let Ok((path, contents)) = process_file(&template, &path) {
                    commit_file(&path, &contents);
                }
            }
        });
    }
    http::server::run();
}

fn main() {
    match cli::args::parse() {
        cli::Config::Build(config) => build(config),
        cli::Config::Serve(config) => serve(config),
    }
}
