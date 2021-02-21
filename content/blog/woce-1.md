+++
title = "Writing our own Cheat Engine: Introduction"
date = 2021-02-07
updated = 2021-02-19
[taxonomies]
category = ["sw"]
tags = ["windows", "rust", "hacking"]
+++

This is part 1 on the *Writing our own Cheat Engine* series:

* Part 1: Introduction
* [Part 2: Exact Value scanning](/blog/woce-2)
* [Part 3: Unknown initial value](/blog/woce-3)

[Cheat Engine][ce] is a tool designed to modify single player games and contains other useful tools within itself that enable its users to debug games or other applications. It comes with a memory scanner, (dis)assembler, inspection tools and a handful other things. In this series, we will be writing our own tiny Cheat Engine capable of solving all steps of the tutorial, and diving into how it all works underneath.

Needless to say, we're doing this for private and educational purposes only. One has to make sure to not violate the EULA or ToS of the specific application we're attaching to. This series, much like cheatengine.org, does not condone the illegal use of the code shared.

Cheat Engine is a tool for Windows, so we will be developing for Windows as well. However, you can also [read memory from Linux-like systems][linux-readmem]. [GameConqueror][game-conqueror] is a popular alternative to Cheat Engine on Linux systems, so if you feel adventurous, you could definitely follow along too! The techniques shown in this series apply regardless of how we read memory from a process. You will learn a fair bit about doing FFI in Rust too.

We will be developing the application in Rust, because it enables us to interface with the Windows API easily, is memory safe (as long as we're careful with `unsafe`!), and is speedy (we will need this for later steps in the Cheat Engine tutorial). You could use any language of your choice though. For example, [Python also makes it relatively easy to use the Windows API][python-ctypes]. You don't need to be a Rust expert to follow along, but this series assumes some familiarity with C-family languages. Slightly advanced concepts like the use of `unsafe` or the `MaybeUninit` type will be briefly explained. What a `fn` is or what `let` does will not be explained.

[Cheat Engine's source code][ce-code] is mostly written in Pascal and C. And it's *a lot* of code, with a very flat project structure, and files ranging in the thousand lines of code each. It's daunting[^1]. It's a mature project, with a lot of knowledge encoded in the code base, and a lot of features like distributed scanning or an entire disassembler. Unfortunately, there's not a lot of comments. For these reasons, I'll do some guesswork when possible as to how it's working underneath, rather than actually digging into what Cheat Engine is actually doing.

With that out of the way, let's get started!

## Welcome to the Cheat Engine Tutorial

<details open><summary>Cheat Engine Tutorial: Step 1</summary>

> This tutorial will teach you the basics of cheating in video games. It will also show you foundational aspects of using Cheat Engine (or CE for short). Follow the steps below to get started.
>
> 1. Open Cheat Engine if it currently isn't running.
> 2. Click on the "Open Process" icon (it's the top-left icon with the computer on it, below "File".).
> 3. With the Process List window now open, look for this tutorial's process in the list. It will look something like > "00001F98-Tutorial-x86_64.exe" or "0000047C-Tutorial-i386.exe". (The first 8 numbers/letters will probably be different.)
> 4. Once you've found the process, click on it to select it, then click the "Open" button. (Don't worry about all the > other buttons right now. You can learn about them later if you're interested.)
>
> Congratulations! If you did everything correctly, the process window should be gone with Cheat Engine now attached to the > tutorial (you will see the process name towards the top-center of CE).
>
> Click the "Next" button below to continue, or fill in the password and click the "OK" button to proceed to that step.)
>
> If you're having problems, simply head over to forum.cheatengine.org, then click on "Tutorials" to view beginner-friendly > guides!

</details>

## Enumerating processes

Our first step is attaching to the process we want to work with. But we need a way to find that process in the first place! Having to open the task manager, look for the process we care about, noting down the process ID (PID), and slapping it in the source code is not satisfying at all. Instead, let's enumerate all the processes from within the program, and let the user select one by typing its name.

From a quick [DuckDuckGo search][ddg-enumproc], we find an official tutorial for [Enumerating All Processes][tut-enumproc], which leads to the [`EnumProcesses`][api-enumproc] call. Cool! Let's slap in the [`winapi`][winapi-crate] crate on `Cargo.toml`, because I don't want to write all the definitions by myself:

```toml
[dependencies]
winapi = { version = "0.3.9", features = ["psapi"] }
```

Because [`EnumProcesses`][api-enumproc] is in `Psapi.h` (you can see this in the online page of its documentation), we know we'll need the `psapi` crate feature. Another option is to search for it in the [`winapi` documentation][winapi-doc] and noting down the parent module where its stored.

The documentation for the method has the following remark:

> It is a good idea to use a large array, because it is hard to predict how many processes there will be at the time you call **EnumProcesses**.

*Sidenote: reading the documentation for the methods we'll use from the Windows API is extremely important. There's a lot of gotchas involved, so we need to make sure we're extra careful.*

1024 is a pretty big number, so let's go with that:

```rust
use std::io;
use std::mem;
use winapi::shared::minwindef::{DWORD, FALSE};

pub fn enum_proc() -> io::Result<Vec<u32>> {
    let mut pids = Vec::<DWORD>::with_capacity(1024);
    let mut size = 0;
    // SAFETY: the pointer is valid and the size matches the capacity.
    if unsafe {
        winapi::um::psapi::EnumProcesses(
            pids.as_mut_ptr(),
            (pids.capacity() * mem::size_of::<DWORD>()) as u32,
            &mut size,
        )
    } == FALSE
    {
        return Err(io::Error::last_os_error());
    }

    todo!()
}
```

We allocate enough space[^2] for 1024 `pids` in a vector[^3], and pass a mutable pointer to the contents to `EnumProcesses`. Note that the size of the array is in *bytes*, not items, so we need to multiply the capacity by the size of `DWORD`. The API likes to use `u32` for sizes, unlike Rust which uses `usize`, so we need a cast.

Last, we need another mutable variable where the amount of bytes written is stored, `size`.

> If the function fails, the return value is zero. To get extended error information, call [`GetLastError`][getlasterr].

That's precisely what we do. If it returns false (zero), we return the last OS error. Rust provides us with [`std::io::Error::last_os_error`][lasterr], which essentially makes that same call but returns a proper `io::Error` instance. Cool!

> To determine how many processes were enumerated, divide the *lpcbNeeded* value by `sizeof(DWORD)`.

Easy enough:

```rust
let count = size as usize / mem::size_of::<DWORD>();
// SAFETY: the call succeeded and count equals the right amount of items.
unsafe { pids.set_len(count) };
Ok(pids)
```

Rust doesn't know that the memory for `count` items were initialized by the call, but we do, so we make use of the [`Vec::set_len`][vecsetlen] call to indicate this. The Rust documentation even includes a FFI similar to our code!

Let's give it a ride:

```rust
fn main() {
    dbg!(enum_proc().unwrap().len());
}
```

```
>cargo run
   Compiling memo v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 0.20s
     Running `target\debug\memo.exe`
[src\main.rs:27] enum_proc().unwrap().len() = 178
```

It works! But currently we only have a bunch of process identifiers, with no way of knowing which process they refer to.

> To obtain process handles for the processes whose identifiers you have just obtained, call the [`OpenProcess`][openproc] function.

Oh!

## Opening a process

The documentation for `OpenProcess` also contains the following:

> When you are finished with the handle, be sure to close it using the [`CloseHandle`](closehandle) function.

This sounds to me like the perfect time to introduce a custom `struct Process` with an `impl Drop`! We're using `Drop` to cleanup resources, not behaviour, so it's fine. [Using `Drop` to cleanup behaviour is a bad idea][drop-behaviour]. But anyway, let's get back to the code:

```rust
use std::ptr::NonNull;
use winapi::ctypes::c_void;

pub struct Process {
    pid: u32,
    handle: NonNull<c_void>,
}

impl Process {
    pub fn open(pid: u32) -> io::Result<Self> {
        todo!()
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        todo!()
    }
}
```

For `open`, we'll want to use `OpenProcess` (and we also need to add the `processthreadsapi` feature to the `winapi` dependency in `Cargo.toml`). It returns a `HANDLE`, which is a nullable mutable pointer to `c_void`. If it's null, the call failed, and if it's non-null, it succeeded and we have a valid handle. This is why we use Rust's [`NonNull`][nonnull]:

```rust
// SAFETY: the call doesn't have dangerous side-effects.
NonNull::new(unsafe { winapi::um::processthreadsapi::OpenProcess(0, FALSE, pid) })
    .map(|handle| Self { pid, handle })
    .ok_or_else(io::Error::last_os_error)
```

`NonNull` will return `Some` if the pointer is non-null. We map the non-null pointer to a `Process` instance with `Self { .. }`. `ok_or_else` converts the `Option` to a `Result` with the error builder function we provide if it was `None`.

The first parameter is a bitflag of permissions we want to have. For now, we can leave it as zero (all bits unset, no specific permissions granted). The second one is whether we want to inherit the handle, which we don't, and the third one is the process identifier. Let's close the resource handle on `Drop` (after adding `handleapi` to the crate features):

```rust
// SAFETY: the handle is valid and non-null.
unsafe { winapi::um::handleapi::CloseHandle(self.handle.as_mut()) };
```

`CloseHandle` can actually fail (for example, on double-close), but given our invariants, it won't. You could add an `assert!` to panic if this is not the case.

We can now open processes, and they will be automatically closed on `Drop`. Does any of this work though?

```rust
fn main() {
    let mut success = 0;
    let mut failed = 0;
    enum_proc().unwrap().into_iter().for_each(|pid| match Process::open(pid) {
        Ok(_) => success += 1,
        Err(_) => failed += 1,
    });

    eprintln!("Successfully opened {}/{} processes", success, success + failed);
}
```

```
>cargo run
   Compiling memo v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 0.36s
     Running `target\debug\memo.exe`
Successfully opened 0/191 processes
```

…nope. Maybe the documentation for `OpenProcess` says something?

> `dwDesiredAccess`
>
> The access to the process object. This access right is checked against the security descriptor for the process. This parameter can be **one or more** of the process access rights.

One or more, but we're setting zero permissions. I told you, reading the documentation is important[^4]! The [Process Security and Access Rights][proc-rights] page lists all possible values we could use. `PROCESS_QUERY_INFORMATION` seems to be appropriated:

> Required to retrieve certain information about a process, such as its token, exit code, and priority class

```rust
OpenProcess(winapi::um::winnt::PROCESS_QUERY_INFORMATION, ...)
```

Does this fix it?

```rust
>cargo run
   Compiling memo v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 0.36s
     Running `target\debug\memo.exe`
Successfully opened 69/188 processes
```

*Nice*. It does solve it. But why did we only open 69 processes out of 188? Does it help if we run our code as administrator? Let's search for `cmd` in the Windows menu and right click to Run as administrator, then `cd` into our project and try again:

```
>cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target\debug\memo.exe`
Successfully opened 77/190 processes
```

We're able to open a few more, so it does help. In general, we'll want to run as administrator, so normal programs can't sniff on what we're doing, and so that we have permission to do more things.

## Getting the name of a process

We're not done enumerating things just yet. To get the "name" of a process, we need to enumerate the modules that it has loaded, and only then can we get the module base name. The first module is the program itself, so we don't need to enumerate *all* modules, just the one is enough.

For this we want [`EnumProcessModules`][mod-enumproc] and [`GetModuleBaseNameA`][mod-name]. I'm using the ASCII variant of `GetModuleBaseName` because I'm too lazy to deal with UTF-16 of the `W` (wide, unicode) variants.

```rust
use std::mem::MaybeUninit;
use winapi::shared::minwindef::HMODULE;

pub fn name(&self) -> io::Result<String> {
    let mut module = MaybeUninit::<HMODULE>::uninit();
    let mut size = 0;
    // SAFETY: the pointer is valid and the size is correct.
    if unsafe {
        winapi::um::psapi::EnumProcessModules(
            self.handle.as_ptr(),
            module.as_mut_ptr(),
            mem::size_of::<HMODULE>() as u32,
            &mut size,
        )
    } == FALSE
    {
        return Err(io::Error::last_os_error());
    }

    // SAFETY: the call succeeded, so module is initialized.
    let module = unsafe { module.assume_init() };
    todo!()
}
```

`EnumProcessModules` takes a pointer to an array of `HMODULE`. We could use a `Vec` of capacity one to hold the single module, but in memory, a pointer a single item can be seen as a pointer to an array of items. `MaybeUninit` helps us reserve enough memory for the one item we need.

With the module handle, we can retrieve its base name:

```rust
let mut buffer = Vec::<u8>::with_capacity(64);
// SAFETY: the handle, module and buffer are all valid.
let length = unsafe {
    winapi::um::psapi::GetModuleBaseNameA(
        self.handle.as_ptr(),
        module,
        buffer.as_mut_ptr().cast(),
        buffer.capacity() as u32,
    )
};
if length == 0 {
    return Err(io::Error::last_os_error());
}

// SAFETY: the call succeeded and length represents bytes.
unsafe { buffer.set_len(length as usize) };
Ok(String::from_utf8(buffer).unwrap())
```

Similar to how we did with `EnumProcesses`, we create a buffer that will hold the ASCII string of the module's base name[^5]. The call wants us to pass a pointer to a mutable buffer of `i8`, but Rust's `String::from_utf8` wants a `Vec<u8>`, so instead we declare a buffer of `u8` and `.cast()` the pointer in the call. You could also do this with `as _`, and Rust would infer the right type, but `cast` is neat.

We `unwrap` the creation of the UTF-8 string because the buffer should contain only ASCII characters (which are also valid UTF-8). We could use the `unsafe` variant to create the string, but what if somehow it contains non-ASCII characters? The less `unsafe`, the better.

Let's see it in action:

```rust
fn main() {
    enum_proc()
        .unwrap()
        .into_iter()
        .for_each(|pid| match Process::open(pid) {
            Ok(proc) => match proc.name() {
                Ok(name) => println!("{}: {}", pid, name),
                Err(e) => println!("{}: (failed to get name: {})", pid, e),
            },
            Err(e) => eprintln!("failed to open {}: {}", pid, e),
        });
}
```

```
>cargo run
   Compiling memo v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 0.32s
     Running `target\debug\memo.exe`
failed to open 0: The parameter is incorrect. (os error 87)
failed to open 4: Access is denied. (os error 5)
...
failed to open 5940: Access is denied. (os error 5)
5608: (failed to get name: Access is denied. (os error 5))
...
1704: (failed to get name: Access is denied. (os error 5))
failed to open 868: Access is denied. (os error 5)
...
```

That's not good. What's up with that? Maybe…

> The handle must have the `PROCESS_QUERY_INFORMATION` and `PROCESS_VM_READ` access rights.

…I should've read the documentation. Okay, fine:

```rust
use winapi::um::winnt;
OpenProcess(winnt::PROCESS_QUERY_INFORMATION | winnt::PROCESS_VM_READ, ...)
```

```
>cargo run
   Compiling memo v0.1.0 (C:\Users\L\Desktop\memo)
    Finished dev [unoptimized + debuginfo] target(s) in 0.35s
     Running `target\debug\memo.exe`
failed to open 0: The parameter is incorrect. (os error 87)
failed to open 4: Access is denied. (os error 5)
...
9348: cheatengine-x86_64.exe
3288: Tutorial-x86_64.exe
8396: cmd.exe
4620: firefox.exe
7964: cargo.exe
10052: cargo.exe
5756: memo.exe
```

Hooray 🎉! There's some processes we can't open, but that's because they're system processes. Security works!

## Finale

That was a fairly long post when all we did was print a bunch of pids and their corresponding name. But in all fairness, we also laid out a good foundation for what's coming next.

You can [obtain the code for this post][code] over at my GitHub. At the end of every post, the last commit will be tagged, so you can `git checkout step1` to see the final code for any blog post.

In the [next post](/blog/woce-2), we'll tackle the second step of the tutorial: Exact Value scanning.

### Footnotes

[^1]: You could say I simply love reinventing the wheel, which I do, but in this case, the codebase contains *far* more features than we're interested in. The (apparent) lack of structure and documentation regarding the code, along with the unfortunate [lack of license][lack-license] for the source code, make it a no-go. There's a license, but I think that's for the distributed program itself.

[^2]: If it turns out that there are more than 1024 processes, our code will be unaware of those extra processes. The documentation suggests to perform the call again with a larger buffer if `count == provided capacity`, but given I have under 200 processes on my system, it seems unlikely we'll reach this limit. If you're worried about hitting this limit, simply use a larger limit or retry with a larger vector.

[^3]: C code would likely use [`GlobalAlloc`][global-alloc] here, but Rust's `Vec` handles the allocation for us, making the code both simpler and more idiomatic. In general, if you see calls to `GlobalAlloc` when porting some code to Rust, you can probably replace it with a `Vec`.

[^4]: This will be a recurring theme.

[^5]: …and similar to `EnumProcesses`, if the name doesn't fit in our buffer, the result will be truncated.

[ce]: https://cheatengine.org/
[python-ctypes]: https://lonami.dev/blog/ctypes-and-windows/
[ce-code]: https://github.com/cheat-engine/cheat-engine/
[linux-readmem]: https://stackoverflow.com/q/12977179/4759433
[game-conqueror]: https://github.com/scanmem/scanmem
[ddg-enumproc]: https://ddg.gg/winapi%20enumerate%20all%20processes
[tut-enumproc]: https://docs.microsoft.com/en-us/windows/win32/psapi/enumerating-all-processes
[api-enumproc]: https://docs.microsoft.com/en-us/windows/win32/api/psapi/nf-psapi-enumprocesses
[winapi-crate]: https://crates.io/crates/winapi
[winapi-doc]: https://docs.rs/winapi/
[getlasterr]: https://docs.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-getlasterror
[lasterr]: https://doc.rust-lang.org/stable/std/io/struct.Error.html#method.last_os_error
[vecsetlen]: https://doc.rust-lang.org/stable/std/vec/struct.Vec.html#method.set_len
[openproc]: https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-openprocess
[closehandle]: https://docs.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-closehandle
[drop-behaviour]: https://internals.rust-lang.org/t/pre-rfc-leave-auto-trait-for-reliable-destruction/13825
[nonnull]: https://doc.rust-lang.org/stable/std/ptr/struct.NonNull.html
[proc-rights]: https://docs.microsoft.com/en-us/windows/win32/procthread/process-security-and-access-rights
[mod-enumproc]: https://docs.microsoft.com/en-us/windows/win32/api/psapi/nf-psapi-enumprocessmodules
[mod-name]: https://docs.microsoft.com/en-us/windows/win32/api/psapi/nf-psapi-getmodulebasenamea
[code]: https://github.com/lonami/memo
[lack-license]: https://github.com/cheat-engine/cheat-engine/issues/60
[global-alloc]: https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-globalalloc
