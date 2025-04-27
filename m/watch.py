import ctypes
import platform
import threading
from enum import IntEnum
from pathlib import Path
from queue import Queue, Empty as QueueEmpty
from typing import Generator

FILE_FLAG_BACKUP_SEMANTICS = 0x02000000
FILE_NOTIFY_CHANGE_DIR_NAME = 0x00000002
FILE_NOTIFY_CHANGE_FILE_NAME = 0x00000001
FILE_NOTIFY_CHANGE_LAST_WRITE = 0x00000010
FILE_SHARE_DELETE = 0x00000004
FILE_SHARE_READ = 0x00000001
FILE_SHARE_WRITE = 0x00000002
GENERIC_READ = 0x80000000
GENERIC_WRITE = 0x40000000
INVALID_HANDLE_VALUE = 0xFFFFFFFFFFFFFFFF
OPEN_EXISTING = 3


class FileAction(IntEnum):
    ADDED = 0x00000001
    REMOVED = 0x00000002
    MODIFIED = 0x00000003
    RENAMED_OLD_NAME = 0x00000004
    RENAMED_NEW_NAME = 0x00000005


class FILE_NOTIFY_INFORMATION(ctypes.Structure):
    _fields_ = [
        ("NextEntryOffset", ctypes.c_int32),
        ("Action", ctypes.c_int32),
        ("FileNameLength", ctypes.c_int32),
        ("FileName", ctypes.c_uint16),
    ]


def iter_notify_action_name(
    buffer: ctypes.Array[ctypes.c_char],
) -> Generator[tuple[int, str], None, None]:
    offset = 0
    while True:
        addr = ctypes.addressof(buffer) + offset
        info = ctypes.cast(addr, ctypes.POINTER(FILE_NOTIFY_INFORMATION))[0]
        yield info.Action, ctypes.wstring_at(
            addr + FILE_NOTIFY_INFORMATION.FileName.offset,
            info.FileNameLength // ctypes.sizeof(ctypes.c_wchar),
        )
        if not info.NextEntryOffset:
            break
        offset = info.NextEntryOffset


def watch_windows(
    path: Path, *, until: threading.Event | None = None
) -> Generator[tuple[FileAction, Path], None, None]:
    until = until or threading.Event()
    queue: Queue[tuple[FileAction, Path]] = Queue()
    fileNotifyBuffer = ctypes.create_string_buffer(1024)
    bytesReturned = ctypes.c_int32()
    dirHandle = ctypes.windll.kernel32.CreateFileW(
        str(path.absolute()),
        GENERIC_READ | GENERIC_WRITE,
        FILE_SHARE_DELETE | FILE_SHARE_READ | FILE_SHARE_WRITE,
        None,
        OPEN_EXISTING,
        FILE_FLAG_BACKUP_SEMANTICS,
        None,
    )
    if dirHandle == INVALID_HANDLE_VALUE:
        exit(ctypes.get_last_error())

    def listener():
        while not until.is_set():
            changeHandle = ctypes.windll.kernel32.ReadDirectoryChangesW(
                dirHandle,
                fileNotifyBuffer,
                len(fileNotifyBuffer),
                True,
                FILE_NOTIFY_CHANGE_FILE_NAME
                | FILE_NOTIFY_CHANGE_DIR_NAME
                | FILE_NOTIFY_CHANGE_LAST_WRITE,
                ctypes.byref(bytesReturned),
                None,
                None,
            )

            if not changeHandle:
                exit(ctypes.get_last_error())

            if bytesReturned:
                for action, name in iter_notify_action_name(fileNotifyBuffer):
                    queue.put((FileAction(action), path / name))

    thread = threading.Thread(target=listener)
    thread.start()

    try:
        while not until.is_set():
            try:
                yield queue.get(timeout=1)
            except QueueEmpty:
                continue  # needing a timeout is a documented problem on Windows
    finally:
        until.set()
        if not ctypes.windll.kernel32.CancelIoEx(dirHandle, None):
            exit(ctypes.get_last_error())

        thread.join()


def watch(path: Path, *, until: threading.Event | None = None):
    if platform.system() == "Windows":
        yield from watch_windows(path, until=until)
    else:
        raise RuntimeError(
            f"file watcher not implemented for system: {platform.system()}"
        )
