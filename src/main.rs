use std::path::{Path, PathBuf};
use std::process;
use std::{fs, thread};

pub mod cli;
pub mod collections;
pub mod conf;
pub mod css;
pub mod date;
pub mod entry;
pub mod feed;
pub mod file_watcher;
pub mod html;
pub mod http;
pub mod markdown;
pub mod template;
pub mod toml;
pub mod walkdir;
#[cfg(target_os = "windows")]
pub mod winapi;
pub mod xml;

use cli::BuildConfig;
use entry::Entry;

fn load_template() -> Vec<u8> {
    let path = PathBuf::from(conf::INPUT_FOLDER).join(conf::TEMPLATE_NAME);
    html::minify(&fs::read(path).expect("path to be a readable file"))
}

fn commit_file(path: &Path, contents: &[u8]) {
    fs::create_dir_all(path.parent().expect("path to have a parent"))
        .expect("parent directories to be created");

    fs::write(path, contents).expect("path to be writable");
}

fn build(config: cli::BuildConfig) {
    let mut entries = Vec::<Entry>::new();

    entries.push(Entry::from_new_path_with_contents(
        PathBuf::from("CNAME"),
        conf::CNAME.as_bytes().to_vec(),
    ));

    let template = load_template();
    for dir_entry in walkdir::walk(PathBuf::from(conf::INPUT_FOLDER)) {
        if dir_entry.file_name() == conf::TEMPLATE_NAME {
            continue;
        }

        match Entry::load_from_path(dir_entry.path()) {
            Ok(entry) => {
                entries.push(entry);
            }
            Err(error) => {
                println!("failed to process file: {:?}\n  {error}", dir_entry.path());
                if config.ignore_errors {
                    continue;
                }
                process::exit(1);
            }
        }
    }

    entries.push(Entry::from_new_path_with_contents(
        PathBuf::from("blog/atom.xml"),
        feed::from_markdown_entries(entries.iter().filter(|entry| {
            entry.path.extension().is_some_and(|e| e == "md")
                && entry.path.file_stem().is_none_or(|s| s != "_index")
                && entry.path_parent() == Some(&PathBuf::from(conf::INPUT_FOLDER).join("blog"))
        }))
        .to_string()
        .into_bytes(),
    ));

    if config.write {
        if config.force {
            let _ = fs::remove_dir_all(&config.output_folder);
        }

        for entry in &entries {
            commit_file(
                &config.output_folder.join(&entry.processed_path),
                &template::apply(&template, &entries, entry),
            );
        }
    }
}

fn deploy() {
    fn run_git_expecting_success_with(
        func: impl Fn(&mut process::Command) -> &mut process::Command,
    ) {
        let mut git = process::Command::new("git");
        git.arg("-C").arg(conf::OUTPUT_FOLDER);
        func(&mut git);
        let status = git.status();
        if !status.expect("process status to be readable").success() {
            println!("executing command failed: {git:?}");
            process::exit(1);
        }
    }

    let origin = process::Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .output()
        .expect("git remote to succeed")
        .stdout;

    build(BuildConfig {
        write: true,
        force: true,
        ignore_errors: false,
        output_folder: PathBuf::from(conf::OUTPUT_FOLDER),
    });

    run_git_expecting_success_with(|c| c.arg("init").arg("--initial-branch").arg("gh-pages"));
    run_git_expecting_success_with(|c| {
        c.arg("remote")
            .arg("add")
            .arg("origin")
            .arg(String::from_utf8_lossy(origin.trim_ascii()).as_ref())
    });
    run_git_expecting_success_with(|c| c.arg("add").arg("."));
    run_git_expecting_success_with(|c| c.arg("commit").arg("--message").arg("site deploy"));
    run_git_expecting_success_with(|c| c.arg("push").arg("--force"));

    fs::remove_dir_all(PathBuf::from(conf::OUTPUT_FOLDER).join(".git"))
        .expect("output folder git directory to be delete-able");
}

fn serve(config: cli::ServeConfig) {
    if config.watch {
        thread::spawn(|| {
            let template = load_template();
            let output_folder = PathBuf::from(conf::OUTPUT_FOLDER);
            for path in file_watcher::watch(conf::INPUT_FOLDER) {
                if let Ok(entry) = Entry::load_from_path(path) {
                    commit_file(
                        &output_folder.join(&entry.processed_path),
                        &template::apply(&template, &[], &entry),
                    );
                }
            }
        });
    }
    http::server::run();
}

fn main() {
    match cli::args::parse() {
        cli::Config::Build(config) => build(config),
        cli::Config::Deploy => deploy(),
        cli::Config::Serve(config) => serve(config),
    }
}
