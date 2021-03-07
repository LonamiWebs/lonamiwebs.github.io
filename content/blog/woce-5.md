+++
title = "Writing our own Cheat Engine: Code finder"
date = 2021-03-06
updated = 2021-03-06
[taxonomies]
category = ["sw"]
tags = ["windows", "rust", "hacking"]
+++

This is part 5 on the *Writing our own Cheat Engine* series:

* [Part 1: Introduction](/blog/woce-1) (start here if you're new to the series!)
* [Part 2: Exact Value scanning](/blog/woce-2)
* [Part 3: Unknown initial value](/blog/woce-3)
* [Part 4: Floating points](/blog/woce-4)
* Part 5: Code finder

In part 4 we spent a good deal of time trying to make our scans generic, and now we have something that works[^1]! Now that the scanning is fairly powerful and all covered, the Cheat Engine tutorial shifts focus into slightly more advanced techniques that you will most certainly need in anything bigger than a toy program.

It's time to write our very own **debugger** in Rust!

## Code finder

<details open><summary>Cheat Engine Tutorial: Step 5</summary>

> Sometimes the location something is stored at changes when you restart the game, or even while you're playing… In that case you can use 2 things to still make a table that works. In this step I'll try to describe how to use the Code Finder function.
>
> The value down here will be at a different location each time you start the tutorial, so a normal entry in the address list wouldn't work. First try to find the address. (You've got to this point so I assume you know how to.)
>
> When you've found the address, right-click the address in Cheat Engine and choose "Find out what writes to this address". A window will pop up with an empty list.
>
> Then click on the Change value button in this tutorial, and go back to Cheat Engine. If everything went right there should be an address with assembler code there now.
>
> Click it and choose the replace option to replace it with code that does nothing. That will also add the code address to the code list in the advanced options window. (Which gets saved if you save your table.)
>
> Click on stop, so the game will start running normal again, and close to close the window. Now, click on Change value, and if everything went right the Next button should become enabled.
>
> Note: When you're freezing the address with a high enough speed it may happen that next becomes visible anyhow

</details>

## Baby steps to debugging

Although I have used debuggers before, I have never had a need to write one myself so it's time for some research.

Searching on DuckDuckGo, I can find entire series to [Writing a Debugger][debug-series]. We would be done by now if only that series wasn't written for Linux. The Windows documentation contains a section called [Creating a Basic Debugger][win-basic-dbg], but as far as I can tell, it only teaches you the [functions][dbg-func] needed to configure the debugging loop. Which mind you, we will need, but in due time.

According to [Writing your own windows debugger in C][dbg-c], the steps needed to write a debugger are:

* [`SuspendThread(proc)`][suspend-thread]. It makes sense that we need to pause all the threads[^2] before messing around with the code the program is executing, or things are very prone to go wrong.
* [`GetThreadContext(proc)`][thread-ctx]. This function retrieves the appropriate context of the specified thread and is highly processor specific. It basically takes a snapshot of all the registers. Think of registers like extremely fast, but also extremely limited, memory the processor uses.
* [`DebugBreakProcess`][dbg-break]. Essentially [writes out the 0xCC opcode][0xcc], `int 3` in assembly, also known as software breakpoint. It's written wherever the Register Instruction Pointer (RIP[^3]) currently points to, so in essence, when the thread resumes, it will immediately [trigger the breakpoint][how-c-brk].
* [`ContinueDebugEvent`][cont-dbg]. Presumably continues debugging.

There are pages documenting [all of the debug events][dbg-events] that our debugger will be able to handle.

Okay, nice! Software breakpoints seem to be done by writing out memory to the region where the program is reading instructions from. We know how to write memory, as that's what all the previous posts have been doing to complete the corresponding tutorial steps. After the breakpoint is executed, all we need to do is [restore the original memory back][how-int3] so that the next time the program executes the code it sees no difference.

But a software breakpoint will halt execution when the code executes the interrupt instruction. This step of the tutorial wants us to find *what writes to a memory location*. Where should we place the breakpoint to detect such location? Writing out the instruction to the memory we want to break in won't do; it's not an instruction, it's just data.

The name may have given it away. If we're talking about software breakpoints, it makes sense that there would exist such a thing as [*hardware* breakpoints][hw-brk]. Because they're tied to the hardware, they're highly processor-specific, but luckily for us, the processor on your usual desktop computer probably has them! Even the [cortex-m] does. The wikipedia page also tells us the name of the thing we're looking for, watchpoints:

> Other kinds of conditions can also be used, such as the reading, writing, or modification of a specific location in an area of memory. This is often referred to as a conditional breakpoint, a data breakpoint, or a watchpoint.

A breakpoint that triggers when a specific memory location is written to is exactly what we need, and [x86 has debug registers D0 to D3 to track memory addresses][x86-dbg-reg]. As far as I can tell, there is no API in specific to mess with the registers. But we don't need any of that! We can just go ahead and [write some assembly by hand][asm-macro] to access these registers. At the time of writing, inline assembly is unstable, so we need a nightly compiler. Run `rustup toolchain install nightly` if you haven't yet, and execute the following code with `cargo +nightly run`:

```rust
#![feature(asm)] // top of the file

fn main() {
    let x: u64 = 123;
    unsafe {
        asm!("mov dr7, {}", in(reg) x);
    }
}

```

`dr7` stands is the [debug control register][dbg-reg], and running this we get…

```
>cargo +nightly run
   Compiling memo v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 0.74s
     Running `target\debug\memo.exe`
error: process didn't exit successfully: `target\debug\memo.exe` (exit code: 0xc0000096, STATUS_PRIVILEGED_INSTRUCTION)
```

…an exception! In all fairness, I have no idea what that code would have done. So maybe the `STATUS_PRIVILEGED_INSTRUCTION` is just trying to protect us. Can we read from the register instead, and see it's default value?

```rust
let x: u64;
unsafe {
    asm!("mov {}, dr7", out(reg) x);
}
assert_eq!(x, 5);
```

```
>cargo +nightly run
...
error: process didn't exit successfully: `target\debug\memo.exe` (exit code: 0xc0000096, STATUS_PRIVILEGED_INSTRUCTION)
```

Nope. Okay, it seems directly reading from or writing to the debug register is a ring-0 thing. Surely there's a way around this. But first we should figure out how to enumerate and pause all the threads.

## Pausing all the threads

It seems there is no straightforward way to enumerate the threads. One has to [create a "toolhelp"][toolhelp] and poll the entries. I won't bore you with the details. Let's add `tlhelp32` to the crate features of `winapi` and try it out:

```rust

#[derive(Debug)]
pub struct Toolhelp {
    handle: winapi::um::winnt::HANDLE,
}

impl Drop for Toolhelp {
    fn drop(&mut self) {
        unsafe { winapi::um::handleapi::CloseHandle(self.handle) };
    }
}

pub fn enum_threads(pid: u32) -> io::Result<Vec<u32>> {
    const ENTRY_SIZE: u32 = mem::size_of::<winapi::um::tlhelp32::THREADENTRY32>() as u32;

    // size_of(dwSize + cntUsage + th32ThreadID + th32OwnerProcessID)
    const NEEDED_ENTRY_SIZE: u32 = 4 * mem::size_of::<DWORD>() as u32;

    // SAFETY: it is always safe to attempt to call this function.
    let handle = unsafe {
        winapi::um::tlhelp32::CreateToolhelp32Snapshot(winapi::um::tlhelp32::TH32CS_SNAPTHREAD, 0)
    };
    if handle == winapi::um::handleapi::INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let toolhelp = Toolhelp { handle };

    let mut result = Vec::new();
    let mut entry = winapi::um::tlhelp32::THREADENTRY32 {
        dwSize: ENTRY_SIZE,
        cntUsage: 0,
        th32ThreadID: 0,
        th32OwnerProcessID: 0,
        tpBasePri: 0,
        tpDeltaPri: 0,
        dwFlags: 0,
    };

    // SAFETY: we have a valid handle, and point to memory we own with the right size.
    if unsafe { winapi::um::tlhelp32::Thread32First(toolhelp.handle, &mut entry) } != FALSE {
        loop {
            if entry.dwSize >= NEEDED_ENTRY_SIZE && entry.th32OwnerProcessID == pid {
                result.push(entry.th32ThreadID);
            }

            entry.dwSize = ENTRY_SIZE;
            // SAFETY: we have a valid handle, and point to memory we own with the right size.
            if unsafe { winapi::um::tlhelp32::Thread32Next(toolhelp.handle, &mut entry) } == FALSE {
                break;
            }
        }
    }

    Ok(result)
}
```

Annoyingly, invalid handles returned by [`CreateToolhelp32Snapshot`][create-snapshot], are `INVALID_HANDLE_VALUE` (which is -1), not null. But that's not a big deal, we simply can't use `NonNull` here. The function ignores the process identifier when using `TH32CS_SNAPTHREAD`, used to include all threads, and we need to compare the process identifier ourselves.

In summary, we create a "toolhelp" (wrapped in a helper `struct` so that whatever happens, `Drop` will clean it up), initialize a thread enntry (with everything but the structure size to zero) and call `Thread32First` the first time, `Thread32Next` subsequent times. It seems to work all fine!

```rust
dbg!(process::enum_threads(pid));
```

```
[src\main.rs:46] process::enum_threads(pid) = Ok(
    [
        10560,
    ],
)
```

According to this, the Cheat Engine tutorial is only using one thread. Good to know. Much like processes, threads need to be opened before we can use them, with [`OpenThread`][open-thread]:

```rust
pub struct Thread {
    tid: u32,
    handle: NonNull<c_void>,
}

impl Thread {
    pub fn open(tid: u32) -> io::Result<Self> {
        // SAFETY: the call doesn't have dangerous side-effects
        NonNull::new(unsafe {
            winapi::um::processthreadsapi::OpenThread(
                winapi::um::winnt::THREAD_SUSPEND_RESUME,
                FALSE,
                tid,
            )
        })
        .map(|handle| Self { tid, handle })
        .ok_or_else(io::Error::last_os_error)
    }

    pub fn tid(&self) -> u32 {
        self.tid
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        unsafe { winapi::um::handleapi::CloseHandle(self.handle.as_mut()) };
    }
}
```

Just your usual RAII pattern. The thread is opened with permission to suspend and resume it. Let's try to pause the handles with [`SuspendThread`][suspend-thread] to make sure that this thread is actually the one we're looking for:

```rust
pub fn suspend(&mut self) -> io::Result<usize> {
    // SAFETY: the handle is valid.
    let ret = unsafe {
        winapi::um::processthreadsapi::SuspendThread(self.handle.as_ptr())
    };
    if ret == -1i32 as u32 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ret as usize)
    }
}

pub fn resume(&mut self) -> io::Result<usize> {
    // SAFETY: the handle is valid.
    let ret = unsafe {
        winapi::um::processthreadsapi::ResumeThread(self.handle.as_ptr())
    };
    if ret == -1i32 as u32 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ret as usize)
    }
}
```

Both suspend and resume return the previous "suspend count". It's kind of like a barrier or semaphore where the thread only runs if the suspend count is zero. Trying it out:

```rust
let mut threads = thread::enum_threads(pid)
    .unwrap()
    .into_iter()
    .map(Thread::open)
    .collect::<Result<Vec<_>, _>>()
    .unwrap();

threads
    .iter_mut()
    .for_each(|thread| {
        println!("Pausing thread {} for 10 seconds…", thread.tid());
        thread.suspend().unwrap();

        std::thread::sleep(std::time::Duration::from_secs(10));

        println!("Wake up, {}!", thread.tid());
        thread.resume().unwrap();
    });
```

If you run this code with the process ID of the Cheat Engine tutorial, you will see that the tutorial window freezes for ten seconds! Because the main and only thread is paused, it cannot process any window events, so it becomes unresponsive. It is now "safe" to mess around with the thread context.

## Setting hardware breakpoints

I'm definitely not the first person to wonder [How to set a hardware breakpoint?][howto-hw-brk]. This is great, because it means I don't need to ask that question myself. It appears we need to change the debug register *via the thread context*.

One has to be careful to use the right context structure. Confusingly enough, [`WOW64_CONTEXT`][wow64-ctx] is 32 bits, not 64. `CONTEXT` alone seems to be the right one:

```rust
pub fn get_context(&self) -> io::Result<winapi::um::winnt::CONTEXT> {
    let context = MaybeUninit::<winapi::um::winnt::CONTEXT>::zeroed();
    // SAFETY: it's a C struct, and all-zero is a valid bit-pattern for the type.
    let mut context = unsafe { context.assume_init() };
    context.ContextFlags = winapi::um::winnt::CONTEXT_ALL;

    // SAFETY: the handle is valid and structure points to valid memory.
    if unsafe {
        winapi::um::processthreadsapi::GetThreadContext(self.handle.as_ptr(), &mut context)
    } == FALSE
    {
        Err(io::Error::last_os_error())
    } else {
        Ok(context)
    }
}
```

Trying it out:

```rust
thread.suspend().unwrap();

let context = thread.get_context().unwrap();
println!("Dr0: {:016x}", context.Dr0);
println!("Dr7: {:016x}", context.Dr7);
println!("Dr6: {:016x}", context.Dr6);
println!("Rax: {:016x}", context.Rax);
println!("Rbx: {:016x}", context.Rbx);
println!("Rcx: {:016x}", context.Rcx);
println!("Rip: {:016x}", context.Rip);
```

```
Dr0: 0000000000000000
Dr7: 0000000000000000
Dr6: 0000000000000000
Rax: 0000000000001446
Rbx: 0000000000000000
Rcx: 0000000000000000
Rip: 00007ffda4259904
```

Looks about right! Hm, I wonder what happens if I use Cheat Engine to add the watchpoint on the memory location we care about?

```
Dr0: 000000000157e650
Dr7: 00000000000d0001
```

Look at that! The debug registers changed! DR0 contains the location we want to watch for writes, and the debug control register DR7 changed. Cheat Engine sets the same values on all threads (for some reason I now see more than one thread printed for the tutorial, not sure what's up with that; maybe the single-thread is the weird one out).

Hmm, what happens if I watch for access instead of write?

```
Dr0: 000000000157e650
Dr7: 00000000000f0001
```

What if I set both?

```
Dr0: 000000000157e650
Dr7: 0000000000fd0005
```

Most intriguing! This was done by telling Cheat Engine to find "what writes" to the address, then "what accesses" the address. I wonder if the order matters?

```
Dr0: 000000000157e650
Dr7: 0000000000df0005
```

"What accesses" and then "what writes" does change it. Very well! We're only concerned in a single breakpoint, so we won't worry about this, but it's good to know that we can inspect what Cheat Engine is doing. It's also interesting to see how Cheat Engine is using hardware breakpoints and not software breakpoints.

For simplicity, our code is going to assume that we're the only ones messing around with the debug registers, and that there will only be a single debug register in use. Make sure to add `THREAD_SET_CONTEXT` to the permissions when opening the thread handle:

```rust
pub fn set_context(&self, context: &winapi::um::winnt::CONTEXT) -> io::Result<()> {
    // SAFETY: the handle is valid and structure points to valid memory.
    if unsafe {
        winapi::um::processthreadsapi::SetThreadContext(self.handle.as_ptr(), context)
    } == FALSE
    {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub fn watch_memory_write(&self, addr: usize) -> io::Result<()> {
    let mut context = self.get_context()?;
    context.Dr0 = addr as u64;
    context.Dr7 = 0x00000000000d0001;
    self.set_context(&context)?;
    todo!()
}
```

If we do this (and temporarily get rid of the `todo!()`), trying to change the value in the Cheat Engine tutorial will greet us with a warm message:

> **Tutorial-x86_64**
>
> External exception 80000004.
>
> Press OK to ignore and risk data corruption.\
> Press Abort to kill the program.
>
> <kbd>OK</kbd> <kbd>Abort</kbd>

There is no debugger attached yet that could possibly handle this exception, so the exception just propagates. Let's fix that.

## Handling debug events

Now that we've succeeded on setting breakpoints, we can actually follow the steps described in [Creating a Basic Debugger][win-basic-dbg]. It starts by saying that we should use [`DebugActiveProcess`][dbg-active] to attach our processor, the debugger, to the process we want to debug, the debuggee. This function lives under the `debugapi` header, so add it to `winapi` features:

```rust
pub struct DebugToken {
    pid: u32,
}

pub fn debug(pid: u32) -> io::Result<DebugToken> {
    if unsafe { winapi::um::debugapi::DebugActiveProcess(pid) } == FALSE {
        return Err(io::Error::last_os_error());
    };
    let token = DebugToken { pid };
    if unsafe { winapi::um::winbase::DebugSetProcessKillOnExit(FALSE) } == FALSE {
        return Err(io::Error::last_os_error());
    };
    Ok(token)
}

impl Drop for DebugToken {
    fn drop(&mut self) {
        unsafe { winapi::um::debugapi::DebugActiveProcessStop(self.pid) };
    }
}
```

Once again, we create a wrapper `struct` with `Drop` to stop debugging the process once the token is dropped. The call to `DebugSetProcessKillOnExit` in our `debug` method ensures that, if our process (the debugger) dies, the process we're debugging (the debuggee) stays alive. We don't want to be restarting the entire Cheat Engine tutorial every time our Rust code crashes!

With the debugger attached, we can wait for debug events. We will put this method inside of `impl DebugToken`, so that the only way you can call it is if you successfully attached to another process:

```rust
impl DebugToken {
    pub fn wait_event(
        &self,
        timeout: Option<Duration>,
    ) -> io::Result<winapi::um::minwinbase::DEBUG_EVENT> {
        let mut result = MaybeUninit::uninit();
        let timeout = timeout
            .map(|d| d.as_millis().try_into().ok())
            .flatten()
            .unwrap_or(winapi::um::winbase::INFINITE);

        // SAFETY: can only wait for events with a token, so the debugger is active.
        if unsafe { winapi::um::debugapi::WaitForDebugEvent(result.as_mut_ptr(), timeout) } == FALSE
        {
            Err(io::Error::last_os_error())
        } else {
            // SAFETY: the call returned non-zero, so the structure is initialized.
            Ok(unsafe { result.assume_init() })
        }
    }
}
```

`WaitForDebugEvent` wants a timeout in milliseconds, so our function lets the user pass the more Rusty `Duration` type. `None` will indicate "there is no timeout", i.e., it's infinite. If the duration is too large to fit in the `u32` (`try_into` fails), it will also be infinite.

If we attach the debugger, set the hardware watchpoint, and modify the memory location from the tutorial, an event with `dwDebugEventCode = 3` will be returned! Now, back to the page with the [Debugging Events][dbg-events]… Gah! It only has the name of the constants, not the values. Well, good thing [docs.rs] has a source view! We can just check the values in the [source code for `winapi`][winapi-dbg-event-src]:

```rust
pub const EXCEPTION_DEBUG_EVENT: DWORD = 1;
pub const CREATE_THREAD_DEBUG_EVENT: DWORD = 2;
pub const CREATE_PROCESS_DEBUG_EVENT: DWORD = 3;
pub const EXIT_THREAD_DEBUG_EVENT: DWORD = 4;
pub const EXIT_PROCESS_DEBUG_EVENT: DWORD = 5;
pub const LOAD_DLL_DEBUG_EVENT: DWORD = 6;
pub const UNLOAD_DLL_DEBUG_EVENT: DWORD = 7;
pub const OUTPUT_DEBUG_STRING_EVENT: DWORD = 8;
pub const RIP_EVENT: DWORD = 9;
```

So, we've got a `CREATE_PROCESS_DEBUG_EVENT`:

> Generated whenever a new process is created in a process being debugged or whenever the debugger begins debugging an already active process. The system generates this debugging event before the process begins to execute in user mode and before the system generates any other debugging events for the new process.

It makes sense that this is our first event. By the way, if you were trying this out with a `sleep` lying around in your code, you may have noticed that the window froze until the debugger terminated. That's because:

> When the system notifies the debugger of a debugging event, it also suspends all threads in the affected process. The threads do not resume execution until the debugger continues the debugging event by using [`ContinueDebugEvent`][cont-dbg].

Let's call `ContinueDebugMethod` but also wait on more than one event and see what happens:

```rust
for _ in 0..10 {
    let event = debugger.wait_event(None).unwrap();
    println!("Got {}", event.dwDebugEventCode);
    debugger.cont(event, true).unwrap();
}
```

```
Got 3
Got 6
Got 6
Got 6
Got 6
Got 6
Got 6
Got 6
Got 6
Got 6
```

That's a lot of `LOAD_DLL_DEBUG_EVENT`. Pumping it up to one hundred and also showing the index we get the following:

```
0. Got 3
1. Got 6
...
40. Got 6
41. Got 2
42. Got 1
43. Got 4
```

In order, we got:

* One `CREATE_PROCESS_DEBUG_EVENT`.
* Forty `LOAD_DLL_DEBUG_EVENT`.
* One `CREATE_THREAD_DEBUG_EVENT`.
* One `EXCEPTION_DEBUG_EVENT`.
* One `EXIT_THREAD_DEBUG_EVENT`.

And, if after all this, you change the value in the Cheat Engine tutorial (thus triggering our watch point), we get `EXCEPTION_DEBUG_EVENT`!

> Generated whenever an exception occurs in the process being debugged. Possible exceptions include attempting to access inaccessible memory, executing breakpoint instructions, attempting to divide by zero, or any other exception noted in Structured Exception Handling.

If we print out all the fields in the [`EXCEPTION_DEBUG_INFO`][exc-dbg-info] structure:

```
Watching writes to 10e3a0 for 10s
First chance: 1
ExceptionCode: 2147483652
ExceptionFlags: 0
ExceptionRecord: 0x0
ExceptionAddress: 0x10002c5ba
NumberParameters: 0
ExceptionInformation: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
```

The `ExceptionCode`, which is `0x80000004`, corresponds with `EXCEPTION_SINGLE_STEP`:

> A trace trap or other single-instruction mechanism signaled that one instruction has been executed.

The `ExceptionAddress` is supposed to be "the address where the exception occurred". Very well! I have already completed this step of the tutorial, and I know the instruction is `mov [rax],edx` (or, as Cheat Engine shows, the bytes `89 10` in hexadecimal). The opcode for the `nop` instruction is `90` in hexadecimal, so if we replace two bytes at this address, we should be able to complete the tutorial.

Note that we also need to flush the instruction cache, as noted in the Windows documentation:

> Debuggers frequently read the memory of the process being debugged and write the memory that contains instructions to the instruction cache. After the instructions are written, the debugger calls the [`FlushInstructionCache`][flush-ins] function to execute the cached instructions.

So we add a new method to `impl Process`:

```rust
/// Flushes the instruction cache.
///
/// Should be called when writing to memory regions that contain code.
pub fn flush_instruction_cache(&self) -> io::Result<()> {
    // SAFETY: the call doesn't have dangerous side-effects.
    if unsafe {
        winapi::um::processthreadsapi::FlushInstructionCache(
            self.handle.as_ptr(),
            ptr::null(),
            0,
        )
    } == FALSE
    {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
```

And write some quick and dirty code to get this done:

```rust
let addr = ...;
println!("Watching writes to {:x} for 10s", addr);
threads.iter_mut().for_each(|thread| {
    thread.watch_memory_write(addr).unwrap();
});
loop {
    let event = debugger.wait_event(None).unwrap();
    if event.dwDebugEventCode == 1 {
        let exc = unsafe { event.u.Exception() };
        if exc.ExceptionRecord.ExceptionCode == 2147483652 {
            let addr = exc.ExceptionRecord.ExceptionAddress as usize;
            match process.write_memory(addr, &[0x90, 0x90]) {
                Ok(_) => eprintln!("Patched [{:x}] with NOP", addr),
                Err(e) => eprintln!("Failed to patch [{:x}] with NOP: {}", addr, e),
            };
            process.flush_instruction_cache().unwrap();
            debugger.cont(event, true).unwrap();
            break;
        }
    }
    debugger.cont(event, true).unwrap();
}
```

Although it seems to work:

```
Watching writes to 15103f0 for 10s
Patched [10002c5ba] with NOP
```

It really doesn't:

> **Tutorial-x86_64**
>
> Access violation.
>
> Press OK to ignore and risk data corruption.\
> Press Abort to kill the program.
>
> <kbd>OK</kbd> <kbd>Abort</kbd>

Did we write memory somewhere we shouldn't? The documentation does mention "segment-relative" and "linear virtual addresses":

> `GetThreadSelectorEntry` returns the descriptor table entry for a specified selector and thread. Debuggers use the descriptor table entry to convert a segment-relative address to a linear virtual address. The `ReadProcessMemory` and `WriteProcessMemory` functions require linear virtual addresses.

But nope! This isn't the problem. The problem is that the `ExceptionRecord.ExceptionAddress` is *after* the execution happened, so it's already 2 bytes beyond where it should be. We were accidentally writing out the first half of the next instruction, which, yeah, could not end good.

So does it work if I do this instead?:

```rust
process.write_memory(addr - 2, &[0x90, 0x90])
//                        ^^^ new
```

This totally does work. Step 5: complete 🎉

## Properly patching instructions

You may not be satisfied at all with our solution. Not only are we hardcoding some magic constants to set hardware watchpoints, we're also relying on knowledge specific to the Cheat Engine tutorial (insofar that we're replacing two bytes worth of instruction with NOPs).

Properly supporting more than one hardware breakpoint, along with supporting different types of breakpoints, is definitely doable. The meaning of the bits for the debug registers is well defined, and you can definitely study that to come up with [something more sophisticated][cpp-many-brk] and support multiple different breakpoints. But for now, that's out of the scope of this series. The tutorial only wants us to use an on-write watchpoint, and our solution is fine and portable for that use case.

However, relying on the size of the instructions is pretty bad. The instructions x86 executes are of variable length, so we can't possibly just look back until we find the previous instruction, or even naively determine its length. A lot of unrelated sequences of bytes are very likely instructions themselves. We need a disassembler. No, we're not writing our own[^4].

Searching on [crates.io] for "disassembler" yields a few results, and the first one I've found is [iced-x86]. I like the name, it has a decent amount of GitHub stars, and it was last updated less than a month ago. I don't know about you, but I think we've just hit a jackpot!

It's quite heavy though, so I will add it behind a feature gate, and users that want it may opt into it:

```toml
[features]
patch-nops = ["iced-x86"]

[dependencies]
iced-x86 = { version = "1.10.3", optional = true }
```

You can make use of it with `cargo run --features=patch-nops`. I don't want to turn this blog post into a tutorial for `iced-x86`, but in essence, we need to make use of its `Decoder`. Here's the plan:

1. Find the memory region corresponding to the address we want to patch.
2. Read the entire region.
3. Decode the read bytes until the instruction pointer reaches our address.
4. Because we just parsed the previous instruction, we know its length, and can be replaced with NOPs.

```rust
#[cfg(feature = "patch-nops")]
pub fn nop_last_instruction(&self, addr: usize) -> io::Result<()> {
    use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, NasmFormatter};

    let region = self
        .memory_regions()
        .into_iter()
        .find(|region| {
            let base = region.BaseAddress as usize;
            base <= addr && addr < base + region.RegionSize
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no matching region found"))?;

    let bytes = self.read_memory(region.BaseAddress as usize, region.RegionSize)?;

    let mut decoder = Decoder::new(64, &bytes, DecoderOptions::NONE);
    decoder.set_ip(region.BaseAddress as _);

    let mut instruction = Instruction::default();
    while decoder.can_decode() {
        decoder.decode_out(&mut instruction);
        if instruction.next_ip() as usize == addr {
            return self
                .write_memory(instruction.ip() as usize, &vec![0x90; instruction.len()])
                .map(drop);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        "no matching instruction found",
    ))
}
```

Pretty straightforward! We can set the "instruction pointer" of the decoder so that it matches with the address we're reading from. The `next_ip` method comes in really handy. Overall, it's a bit inefficient, because we could reuse the regions retrieved previously, but other than that, there is not much room for improvement.

With this, we are no longer hardcoding the instruction size or guessing which instruction is doing what. You may wonder, what if the region does not start with valid executable code? It could be possible that the instructions are in some memory region with garbage except for a very specific location with real code. I don't know how Cheat Engine handles this, but I think it's reasonable to assume that the region starts with valid code.

As far as I can tell (after having asked a bit around), the encoding is usually self synchronizing (similar to UTF-8), so eventually we should end up with correct instructions. But someone can still intentionally write real code between garbage data which we would then disassemble incorrectly. This is a problem on all variable-length ISAs. Half a solution is to [start at the entry point][howto-disasm], decode all instructions, and follow the jumps. The other half would be correctly identifying jumps created just to trip a disassembler up, and jumps pointing to dynamically-calculated addresses!

## Finale

That was quite a deep dive! We have learnt about the existence of the various breakpoint types (software, hardware, and even behaviour, such as watchpoints), how to debug a separate process, and how to correctly update the code other process is running on-the-fly. The [code for this post][code] is available over at my GitHub. You can run `git checkout step5` after cloning the repository to get the right version of the code.

Although we've only talked about *setting* breakpoints, there are of course [ways of detecting them][detect-brk]. There's [entire guides about it][detect-brk-guide]. Again, we currently hardcode the fact we want to add a single watchpoint using the first debug register. A proper solution here would be to actually calculate the needs that need to be set, as well as keeping track of how many breakpoints have been added so far.

Hardware breakpoints are also limited, since they're simply a bunch of registers, and our machine does not have infinite registers. How are other debuggers like `gdb` able to create a seemingly unlimited amount of breakpoints? Well, the GDB wiki actually has a page on [Internals Watchpoints][gdb-watchpoints], and it's really interesting! `gdb` essentially single-steps through the entire program and tests the expressions after every instruction:

> Software watchpoints are very slow, since GDB needs to single-step the program being debugged and test the value of the watched expression(s) after each instruction.

However, that's not the only way. One could [change the protection level][change-prot] of the region of interest (for example, remove the write permission), and when the program tries to write there, it will fail! In any case, the GDB wiki is actually a pretty nice resource. It also has a section on [Breakpoint Handling][gdb-breakpoints], which contains some additional insight.

With regards to code improvements, `DebugToken::wait_event` could definitely be both nicer and safer to use, with a custom `enum`, so the user does not need to rely on magic constants or having to resort to `unsafe` access to get the right `union` variant.

In the next post, we'll tackle the sixth step of the tutorial: Pointers. It reuses the debugging techniques presented here to backtrack where the pointer for our desired value is coming from, so here we will need to actually *understand* what the instructions are doing, not just patching them out!

### Footnotes

[^1]: I'm not super happy about the design of it all, but we won't actually need anything beyond scanning for integers for the rest of the steps so it doesn't really matter.

[^2]: There seems to be a way to pause the entire process in one go, with the [undocumented `NtSuspendProcess`] function!

[^3]: It really is called that. The naming went from "IP" (instruction pointer, 16 bits), to "EIP" (extended instruction pointer, 32 bits) and currently "RIP" (64 bits). The naming convention for upgraded registers is the same (RAX, RBX, RCX, and so on). The [OS Dev wiki][osdev-wiki] is a great resource for this kind of stuff.

[^4]: Well, we don't need an entire disassembler. Knowing the length of each instruction is enough, but that on its own is also a lot of work.

[debug-series]: http://system.joekain.com/debugger/
[win-basic-dbg]: https://docs.microsoft.com/en-us/windows/win32/debug/creating-a-basic-debugger
[dbg-func]: https://docs.microsoft.com/en-us/windows/win32/debug/debugging-functions
[dbg-c]: https://www.gironsec.com/blog/2013/12/writing-your-own-debugger-windows-in-c/
[suspend-thread]: https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-suspendthread
[thread-ctx]: https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getthreadcontext
[dbg-break]: https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-debugbreakprocess
[0xcc]: https://docs.microsoft.com/en-us/windows-hardware/drivers/debugger/x86-instructions#miscellaneous
[how-c-brk]: https://stackoverflow.com/q/3915511/
[cont-dbg]: https://docs.microsoft.com/en-us/windows/win32/api/debugapi/nf-debugapi-continuedebugevent
[dbg-events]: https://docs.microsoft.com/en-us/windows/win32/debug/debugging-events
[how-int3]: https://stackoverflow.com/q/3747852/
[hw-brk]: https://en.wikipedia.org/wiki/Breakpoint#Hardware
[cortex-m]: https://interrupt.memfault.com/blog/cortex-m-breakpoints
[x86-dbg-reg]: https://stackoverflow.com/a/19109153/
[asm-macro]: https://doc.rust-lang.org/stable/unstable-book/library-features/asm.html
[dbg-reg]: https://en.wikipedia.org/wiki/X86_debug_register
[toolhelp]: https://stackoverflow.com/a/1206915/
[create-snapshot]: https://docs.microsoft.com/en-us/windows/win32/api/tlhelp32/nf-tlhelp32-createtoolhelp32snapshot
[open-thread]: https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-openthread
[howto-hw-brk]: https://social.msdn.microsoft.com/Forums/en-US/0cb3360d-3747-42a7-bc0e-668c5d9ee1ee/how-to-set-a-hardware-breakpoint
[wow64-ctx]: https://stackoverflow.com/q/17504174/
[dbg-active]: https://docs.microsoft.com/en-us/windows/win32/api/debugapi/nf-debugapi-debugactiveprocess
[docs.rs]: https://docs.rs/
[winapi-dbg-event-src]: https://docs.rs/winapi/0.3.9/src/winapi/um/minwinbase.rs.html#203-211
[cont-dbg]: https://docs.microsoft.com/en-us/windows/win32/api/debugapi/nf-debugapi-continuedebugevent
[exc-dbg-info]: https://docs.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-exception_debug_info
[flush-ins]: https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-flushinstructioncache
[crates.io]: https://crates.io
[iced-x86]: https://crates.io/crates/iced-x86
[detect-brk]: https://reverseengineering.stackexchange.com/a/16547
[detect-brk-guide]: https://www.codeproject.com/Articles/30815/An-Anti-Reverse-Engineering-Guide
[gdb-watchpoints]: https://sourceware.org/gdb/wiki/Internals%20Watchpoints
[gdb-breakpoints]: https://sourceware.org/gdb/wiki/Internals/Breakpoint%20Handling
[howto-disasm]: https://stackoverflow.com/q/3983735/
[cpp-many-brk]: https://github.com/mmorearty/hardware-breakpoints
[change-prot]: https://stackoverflow.com/a/7805842/
[suspend-proc]: https://stackoverflow.com/a/4062698/
[osdev-wiki]: https://wiki.osdev.org/CPU_Registers_x86_64
[code]: https://github.com/lonami/memo
