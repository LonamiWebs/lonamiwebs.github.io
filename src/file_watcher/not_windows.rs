use std::path::PathBuf;

pub struct DirectoryWatcher {}

impl Iterator for DirectoryWatcher {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

pub fn watch(folder: &str) -> DirectoryWatcher {
    unimplemented!()
}
