use std::path::{Path, PathBuf};
use std::{io, process, ptr, slice};

use crate::winapi;

fn exit_with_last_error() {
    let error = io::Error::last_os_error();
    eprintln!("fatal ffi error: {error}");
    process::exit(1);
}

fn open_folder(file: &Path) -> isize {
    let file = file.canonicalize().expect("folder to exist");

    let file_name = file.to_string_lossy().encode_utf16().collect::<Vec<_>>();

    let dir_handle = unsafe {
        winapi::CreateFileW(
            file_name.as_ptr(),
            winapi::GENERIC_READ | winapi::GENERIC_WRITE,
            winapi::FILE_SHARE_DELETE | winapi::FILE_SHARE_READ | winapi::FILE_SHARE_WRITE,
            ptr::null(),
            winapi::OPEN_EXISTING,
            winapi::FILE_FLAG_BACKUP_SEMANTICS,
            0,
        )
    };
    if dir_handle == winapi::INVALID_HANDLE_VALUE {
        exit_with_last_error();
    }
    dir_handle
}

fn poll_directory_change(dir_handle: isize, buffer: &mut Vec<u8>, result: &mut Vec<(i32, String)>) {
    let mut read = 0;
    let change_handle = unsafe {
        winapi::ReadDirectoryChangesW(
            dir_handle,
            buffer.as_mut_ptr() as _,
            buffer.capacity() as _,
            1,
            winapi::FILE_NOTIFY_CHANGE_FILE_NAME
                | winapi::FILE_NOTIFY_CHANGE_DIR_NAME
                | winapi::FILE_NOTIFY_CHANGE_LAST_WRITE,
            &mut read,
            ptr::null_mut(),
            ptr::null(),
        )
    };
    if change_handle == 0 {
        exit_with_last_error();
    }

    if read > 0 {
        unsafe {
            buffer.set_len(read as _);
        }
    }

    parse_file_notify_informations(buffer, result);
}

fn parse_file_notify_informations(buffer: &[u8], result: &mut Vec<(i32, String)>) {
    let mut offset = 0;
    loop {
        let file_notify_information = unsafe {
            &*(buffer.as_ptr().byte_offset(offset) as *const winapi::FILE_NOTIFY_INFORMATION)
        };

        let action = file_notify_information.Action;
        let file_name = unsafe {
            slice::from_raw_parts(
                file_notify_information.FileName.as_ptr(),
                (file_notify_information.FileNameLength as usize) / size_of::<u16>(),
            )
        };
        let file_name = String::from_utf16_lossy(file_name);
        result.push((action, file_name));

        if file_notify_information.NextEntryOffset == 0 {
            break;
        }
        offset = file_notify_information.NextEntryOffset as _;
    }
}

pub struct DirectoryWatcher {
    root: PathBuf,
    dir_handle: isize,
    buffer: Vec<u8>,
    result: Vec<(i32, String)>,
}

impl Iterator for DirectoryWatcher {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((_, file_name)) = self.result.pop() {
                return Some(self.root.join(file_name));
            }

            poll_directory_change(self.dir_handle, &mut self.buffer, &mut self.result);
            self.result.dedup();
        }
    }
}

pub fn watch(folder: &str) -> DirectoryWatcher {
    let root = PathBuf::from(folder);
    let dir_handle = open_folder(&root);

    DirectoryWatcher {
        root,
        dir_handle,
        buffer: Vec::with_capacity(1024),
        result: Vec::new(),
    }
}
