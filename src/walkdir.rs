use std::fs;
use std::path::PathBuf;

pub struct WalkDir {
    current: Option<fs::ReadDir>,
    pending: Vec<PathBuf>,
}

impl Iterator for WalkDir {
    type Item = fs::DirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(current) = self.current.as_mut() {
                for entry in current.by_ref() {
                    let entry = entry.expect("dir entry to be accessible");
                    let file_type = entry.file_type().expect("file type to be accessible");
                    if file_type.is_dir() {
                        self.pending.push(entry.path());
                    } else if file_type.is_file() {
                        return Some(entry);
                    }
                }
                self.current = None;
            } else if let Some(dir) = self.pending.pop() {
                self.current = Some(fs::read_dir(dir).expect("dir to be readable"));
            } else {
                break None;
            }
        }
    }
}

pub fn walk(dir: PathBuf) -> WalkDir {
    WalkDir {
        current: None,
        pending: vec![dir],
    }
}
