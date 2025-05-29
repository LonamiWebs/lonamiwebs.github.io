use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, io};
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

fn deploy(config: cli::DeployConfig) {
    fn exec_expect_success(command: &mut process::Command) {
        let status = command.status();
        if !status.expect("process status to be readable").success() {
            println!("executing command failed: {command:?}");
            process::exit(1);
        }
    }

    let current_exe = env::current_exe().expect("self-executable path to be accessible");
    match config.token {
        None => {
            let output = process::Command::new("git")
                .arg("status")
                .arg("--porcelain=v1")
                .output()
                .expect("git status to succeed");

            if !output.stdout.is_empty() {
                // This is CRITICAL: forgetting has costed me the draft of an entire blog post.
                println!("tree is dirty and deploying would risk losing data; aborting");
                process::exit(1);
            }

            let token = format!(
                "site-lonami.dev.{:010}",
                SystemTime::now() // "random" number
                    .duration_since(UNIX_EPOCH)
                    .expect("system time to be after epoch")
                    .as_nanos() as u32
            );
            let tmp_root = env::temp_dir().join(&token);
            fs::create_dir(&tmp_root).expect("temporary directory to be created");

            let current_name = current_exe
                .file_name()
                .expect("self-executable file name to be accessible");
            let replicated_exe = tmp_root.join(current_name);
            fs::copy(&current_exe, &replicated_exe)
                .expect("self-executable to be copyable to the temporary directory");

            #[allow(clippy::zombie_processes)]
            process::Command::new(replicated_exe)
                .arg("deploy")
                .arg(token)
                .current_dir(
                    PathBuf::from(".")
                        .canonicalize()
                        .expect("current directory to be accessible"),
                )
                .spawn() // current process must exit, leaving the zombie, for self to be deleted
                .expect("replicated executable to spawn");
        }
        Some(token) => {
            let parent = current_exe
                .parent()
                .expect("self-executable to have parent");
            if !parent.starts_with(env::temp_dir()) {
                println!(
                    "expected replicated executable to be in the temporary directory: {parent:?}"
                );
                process::exit(1);
            }
            let parent_name = parent
                .file_name()
                .expect("self-executable parent to have name")
                .to_string_lossy();

            if parent_name != token {
                println!("expected token to match replicated executable parent directory name");
                process::exit(1);
            }

            build(BuildConfig {
                write: true,
                force: true,
                ignore_errors: false,
                output_folder: parent.to_owned(),
            });

            exec_expect_success(process::Command::new("git").arg("checkout").arg("gh-pages"));

            for file in fs::read_dir(".").expect("current directory to be readable") {
                let file = file.expect("operating on listed files to succeed");
                let file_name = file.file_name().to_string_lossy().into_owned();
                if file_name.starts_with(".") {
                    // This is CRITICAL: forgetting to exclude `.git` has costed me several days worth of work and over 30 commits.
                    continue;
                }

                let path = file.path();
                match fs::remove_file(&path) {
                    Ok(_) => {}
                    Err(ef) => {
                        if ef.kind() == io::ErrorKind::PermissionDenied {
                            match fs::remove_dir_all(&path) {
                                Ok(_) => {}
                                Err(ed) => {
                                    println!(
                                        "failed to remove path ({path:?}) as both file:\n    {ef}\n  ...and directory:\n    {ed}"
                                    );
                                    println!("note: repository has been left in a dirty state");
                                    process::exit(1);
                                }
                            }
                        }
                    }
                }
            }

            for file in fs::read_dir(parent).expect("parent directory to be readable") {
                let file = file.expect("operating on generated files to succeed");
                let path = file.path();
                if path == current_exe {
                    continue;
                }
                fs::rename(file.path(), file.file_name())
                    .expect("moving file to working directory to succeed");
            }

            exec_expect_success(process::Command::new("git").arg("add").arg("."));
            exec_expect_success(
                process::Command::new("git")
                    .arg("commit")
                    .arg("--amend")
                    .arg("--message")
                    .arg("site deploy"),
            );
            exec_expect_success(
                process::Command::new("git")
                    .arg("push")
                    .arg("--force-with-lease"),
            );
            exec_expect_success(process::Command::new("git").arg("checkout").arg("master"));
        }
    }
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
        cli::Config::Deploy(config) => deploy(config),
        cli::Config::Serve(config) => serve(config),
    }
}
