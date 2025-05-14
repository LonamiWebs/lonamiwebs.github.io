#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use std::ffi::c_void;

pub const FILE_FLAG_BACKUP_SEMANTICS: i32 = 0x02000000;
pub const FILE_NOTIFY_CHANGE_DIR_NAME: i32 = 0x00000002;
pub const FILE_NOTIFY_CHANGE_FILE_NAME: i32 = 0x00000001;
pub const FILE_NOTIFY_CHANGE_LAST_WRITE: i32 = 0x00000010;
pub const FILE_SHARE_DELETE: i32 = 0x00000004;
pub const FILE_SHARE_READ: i32 = 0x00000001;
pub const FILE_SHARE_WRITE: i32 = 0x00000002;
pub const GENERIC_READ: i32 = 0x80000000u32 as i32;
pub const GENERIC_WRITE: i32 = 0x40000000;
pub const INVALID_HANDLE_VALUE: isize = 0xFFFFFFFFFFFFFFFFusize as isize;
pub const OPEN_EXISTING: i32 = 3;

#[repr(i32)]
pub enum FILE_ACTION {
    ADDED = 0x00000001,
    REMOVED = 0x00000002,
    MODIFIED = 0x00000003,
    RENAMED_OLD_NAME = 0x00000004,
    RENAMED_NEW_NAME = 0x00000005,
}

#[repr(C)]
pub struct FILE_NOTIFY_INFORMATION {
    pub NextEntryOffset: i32,
    pub Action: i32,
    pub FileNameLength: i32,
    pub FileName: [u16; 0],
}

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn CreateFileW(
        lpFileName: *const u16,
        dwDesiredAccess: i32,
        dwShareMode: i32,
        lpSecurityAttributes: *const c_void,
        dwCreationDisposition: i32,
        dwFlagsAndAttributes: i32,
        hTemplateFile: isize,
    ) -> isize;

    pub fn ReadDirectoryChangesW(
        hDirectory: isize,
        lpBuffer: *mut FILE_NOTIFY_INFORMATION,
        nBufferLength: i32,
        bWatchSubtree: i32,
        dwNotifyFilter: i32,
        lpBytesReturned: *mut i32,
        lpOverlapped: *mut c_void,
        lpCompletionRoutine: *const c_void,
    ) -> i32;

    pub fn CancelIoEx(hFile: isize, lpOverlapped: *const c_void) -> i32;
}
