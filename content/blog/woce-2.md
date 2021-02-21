+++
title = "Writing our own Cheat Engine: Exact Value scanning"
date = 2021-02-12
updated = 2021-02-19
[taxonomies]
category = ["sw"]
tags = ["windows", "rust", "hacking"]
+++

This is part 2 on the *Writing our own Cheat Engine* series:

* [Part 1: Introduction](/blog/woce-1) (start here if you're new to the series!)
* Part 2: Exact Value scanning
* [Part 3: Unknown initial value](/blog/woce-3)

In the introduction, we spent a good deal of time enumerating all running processes just so we could find out the pid we cared about. With the pid now in our hands, we can do pretty much anything to its corresponding process.

It's now time to read the process' memory and write to it. If our process was a single-player game, this would enable us to do things like setting a very high value on the player's current health pool, making us invincible. This technique will often not work for multi-player games, because the server likely knows your true current health (the most you could probably do is make the client render an incorrect value). However, if the server is crappy and it trusts the client, then you're still free to mess around with your current health.

Even if we don't want to write to the process' memory, reading is still very useful. Maybe you could enhance your experience by making a custom overlay that displays useful information, or something that makes noise if it detects the life is too low, or even simulating a keyboard event to automatically recover some mana when you're running low.

Be warned about anti-cheat systems. Anything beyond a basic game is likely to have some protection measures in place, making the analysis more difficult (perhaps the values are scrambled in memory), or even pinging the server if it detects something fishy.

**I am not responsible for any bans!** Use your brain before messing with online games, and don't ruin the fun for everyone else. If you get caught for cheating, I don't want to know about it.

Now that all [script kiddies][script-kid] have left the room, let's proceed with the post.

## Exact Value scanning

<details open><summary>Cheat Engine Tutorial: Step 2</summary>

> Now that you have opened the tutorial with Cheat Engine let's get on with the next step.
>
> You can see at the bottom of this window is the text Health: xxx. Each time you click 'Hit me' your health gets decreased.
>
> To get to the next step you have to find this value and change it to 1000
>
> To find the value there are different ways, but I'll tell you about the easiest, 'Exact Value': First make sure value type is set to at least 2-bytes or 4-bytes. 1-byte will also work, but you'll run into an easy to fix problem when you've found the address and want to change it. The 8-byte may perhaps works if the bytes after the address are 0, but I wouldn't take the bet. Single, double, and the other scans just don't work, because they store the value in a different way.
>
> When the value type is set correctly, make sure the scantype is set to 'Exact Value'. Then fill in the number your health is in the value box. And click 'First Scan'. After a while (if you have a extremely slow pc) the scan is done and the results are shown in the list on the left
>
> If you find more than 1 address and you don't know for sure which address it is, click 'Hit me', fill in the new health value into the value box, and click 'Next Scan'. Repeat this until you're sure you've found it. (that includes that there's only 1 address in the list.....)
>
> Now double click the address in the list on the left. This makes the address pop-up in the list at the bottom, showing you the current value. Double click the value, (or select it and press enter), and change the value to 1000.
>
> If everything went ok the next button should become enabled, and you're ready for the next step.
>
> Note: If you did anything wrong while scanning, click "New Scan" and repeat the scanning again. Also, try playing around with the value and click 'hit me'

</details>

## Our First Scan

The Cheat Engine tutorial talks about "value types" and "scan types" like "exact value".

The **value types** will help us narrow down *what* we're looking for. For example, the integer type `i32` is represented in memory as 32 bits, or 4 bytes. However, `f32` is *also* represented by 4 bytes, and so is `u32`. Or perhaps the 4 bytes represent RGBA values of a color! So any 4 bytes in memory can be interpreted in many ways, and it's up to us to decide which way we interpret the bytes in.

When programming, numbers which are 32-bit wide are common, as they're a good (and fast) size to work with. Scanning for this type is often a good bet. For positive numbers, `i32` is represented the same as `u32` in memory, so even if the value turns out to not be signed, the scan is likely to work. Focusing on `i32` will save us from scanning for `f32` or even other types, like interpreting 8 bytes for `i64`, `f64`, or less bytes like `i16`.

The **scan types** will help us narrow down *how* we're looking for a value. Scanning for an exact value means what you think it does: interpret all 4 bytes in the process' memory as our value type, and check if they exactly match our value. This will often yield a lot of candidates, but it will be enough to get us started. Variations of the exact scan include checking for all values below a threshold, above, in between, or even just… unknown.

What's the point of scanning for unknown values if *everything* in memory is unknown? Sometimes you don't have a concrete value. Maybe your health pool is a bar and it nevers tell you how much health you actually have, just a visual indicator of your percentage left, even if the health is not stored as a percentage. As we will find later on, scanning for unknown values is more useful than it might appear at first.

We can access the memory of our own program by guessing random pointers and trying to read from them. But Windows isolates the memory of each program, so no pointer we could ever guess will let us read from the memory of another process. Luckily for us, searching for "read process memory winapi" leads us to the [`ReadProcessMemory`][readmem] function. Spot on.

```rust
pub fn read_memory(&self, addr: usize, n: usize) -> io::Result<Vec<u8>> {
    todo!()
}
```

Much like trying to dereference a pointer pointing to released memory or even null, reading from an arbitrary address can fail for the same reasons (and more). We will want to signal this with `io::Result`. It's funny to note that, even though we're doing something that seems wildly unsafe (reading arbitrary memory, even if the other process is mutating it at the same time), the function is perfectly safe. If we cannot read something, it will return `Err`, but if it succeeds, it has taken a snapshot of the memory of the process, and the returned value will be correctly initialized.

The function will be defined inside our `impl Process`, since it conveniently holds an open handle to the process in question. It takes `&self`, because we do not need to mutate anything in the `Process` instance. After adding the `memoryapi` feature to `Cargo.toml`, we can perform the call:

```rust
let mut buffer = Vec::<u8>::with_capacity(n);
let mut read = 0;

// SAFETY: the buffer points to valid memory, and the buffer size is correctly set.
if unsafe {
    winapi::um::memoryapi::ReadProcessMemory(
        self.handle.as_ptr(),
        addr as *const _,
        buffer.as_mut_ptr().cast(),
        buffer.capacity(),
        &mut read,
    )
} == FALSE
{
    Err(io::Error::last_os_error())
} else {
    // SAFETY: the call succeeded and `read` contains the amount of bytes written.
    unsafe { buffer.set_len(read as usize) };
    Ok(buffer)
}
```

Great! But the address space is somewhat large. 64 bits large. Eighteen quintillion, four hundred and forty-six quadrillion, seven hundred and forty-four trillion, seventy-three billion, seven hundred and nine million, five hundred and fifty-one thousand, six hundred and sixteen[^1] large. You gave up reading that, didn't you? Anyway, 18'446'744'073'709'551'616 is a *big* number.

I am not willing to wait for the program to scan over so many values. I don't even have 16 [exbibytes] of RAM installed on my laptop yet[^2]! What's up with that?

## Memory regions

The program does not actually have all that memory allocated (surprise!). Random-guessing an address is extremely likely to point out to invalid memory. Reading from the start of the address space all the way to the end would not be any better. And we **need** to do better.

We need to query for the memory regions allocated to the program. For this purpose we can use [`VirtualQueryEx`][vquery].

> Retrieves information about a range of pages within the virtual address space of a specified process.

We have enumerated things before, and this function is not all that different.

```rust
fn memory_regions(&self) -> io::Result<winapi::um::winnt::MEMORY_BASIC_INFORMATION> {
    let mut info = MaybeUninit::uninit();

    // SAFETY: the info structure points to valid memory.
    let written = unsafe {
        winapi::um::memoryapi::VirtualQueryEx(
            self.handle.as_ptr(),
            std::ptr::null(),
            info.as_mut_ptr(),
            mem::size_of::<winapi::um::winnt::MEMORY_BASIC_INFORMATION>(),
        )
    };
    if written == 0 {
        Err(io::Error::last_os_error())
    } else {
        // SAFETY: a non-zero amount was written to the structure
        Ok(unsafe { info.assume_init() })
    }
}
```

We start with a base address of zero[^3] (`std::ptr::null()`), and ask the function to tell us what's in there. Let's try it out, with the `impl-debug` crate feature in `Cargo.toml`:

```rust
dbg!(process.memory_regions());
```

```
>cargo run
Compiling memo v0.1.0

error[E0277]: `winapi::um::winnt::MEMORY_BASIC_INFORMATION` doesn't implement `std::fmt::Debug`
   --> src\main.rs:185:5
    |
185 |     dbg!(process.memory_regions());
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `winapi::um::winnt::MEMORY_BASIC_INFORMATION` cannot be formatted using `{:?}` because it doesn't implement `std::fmt::Debug`
```

That's annoying. It seems not everything has an `impl std::fmt::Debug`, and [you're supposed to send a PR][prdebug] if you want it to have debug, even if the `impl-debug` feature is set. I'm surprised they don't auto-generate all of this and have to rely on manually adding `Debug` as needed? Oh well, let's get rid of the feature and print it out ourselves:

```
eprintln!(
    "Region:
    BaseAddress: {:?}
    AllocationBase: {:?}
    AllocationProtect: {:?}
    RegionSize: {:?}
    State: {:?}
    Protect: {:?}
    Type: {:?}",
    region.BaseAddress,
    region.AllocationBase,
    region.AllocationProtect,
    region.RegionSize,
    region.State,
    region.Protect,
    region.Type,
);
```

Hopefully we don't need to do this often:

```
>cargo run
   Compiling memo v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 0.60s
     Running `target\debug\memo.exe`

Region:
    BaseAddress: 0x0
    AllocationBase: 0x0
    AllocationProtect: 0
    RegionSize: 65536
    State: 65536
    Protect: 1
    Type: 0
```

Awesome! There is a region at `null`, and the `AllocationProtect` of zero indicates that "the caller does not have access" when the region was created. However, `Protect` is `1`, and that is the *current* protection level. A value of one indicates [`PAGE_NOACCESS`][memprot]:

> Disables all access to the committed region of pages. An attempt to read from, write to, or execute the committed region results in an access violation.

Now that we know that the first region starts at 0 and has a size of 64 KiB, we can simply query for the page at `(current base + current size)` to fetch the next region. Essentially, we want to loop until it fails, after which we'll know there are no more pages[^4]:

```rust
pub fn memory_regions(&self) -> Vec<winapi::um::winnt::MEMORY_BASIC_INFORMATION> {
    let mut base = 0;
    let mut regions = Vec::new();
    let mut info = MaybeUninit::uninit();

    loop {
        // SAFETY: the info structure points to valid memory.
        let written = unsafe {
            winapi::um::memoryapi::VirtualQueryEx(
                self.handle.as_ptr(),
                base as *const _,
                info.as_mut_ptr(),
                mem::size_of::<winapi::um::winnt::MEMORY_BASIC_INFORMATION>(),
            )
        };
        if written == 0 {
            break regions;
        }
        // SAFETY: a non-zero amount was written to the structure
        let info = unsafe { info.assume_init() };
        base = info.BaseAddress as usize + info.RegionSize;
        regions.push(info);
    }
}
```

`RegionSize` is:

> The size of the region beginning at the base address in which all pages have identical attributes, in bytes.

…which also hints that the value we want is "base address", not the "allocation base". With these two values, we can essentially iterate over all the page ranges:

```rust
dbg!(process.memory_regions().len());
```

```
>cargo run
   Compiling memo v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 0.63s
     Running `target\debug\memo.exe`

[src\main.rs:189] process.memory_regions().len() = 367
```

That's a lot of pages!

## Protection levels

Let's try to narrow the amount of pages down. How many pages aren't `PAGE_NOACCESS`?

```rust
dbg!(process
    .memory_regions()
    .into_iter()
    .filter(|p| p.Protect != winapi::um::winnt::PAGE_NOACCESS)
    .count());
```

```
295
```

Still a fair bit! Most likely, there are just a few interleaved `NOACCESS` pages, and the rest are allocated each with different protection levels. How much memory do we need to scan through?

```rust
dbg!(process
    .memory_regions()
    .into_iter()
    .filter(|p| p.Protect != winapi::um::winnt::PAGE_NOACCESS)
    .map(|p| p.RegionSize)
    .sum::<usize>());
```

```
4480434176
```

Wait, what? What do you mean over 4 GiB? The Task Manager claims that the Cheat Engine Tutorial is only using 2.1 MB worth of RAM! Perhaps we can narrow down the [protection levels][memprot] a bit more. If you look at the scan options in Cheat Engine, you will notice the "Memory Scan Options" groupbox. By default, it only scans for memory that is writable, and doesn't care if it's executable or not:


```rust
let mask = winnt::PAGE_EXECUTE_READWRITE
    | winnt::PAGE_EXECUTE_WRITECOPY
    | winnt::PAGE_READWRITE
    | winnt::PAGE_WRITECOPY;

dbg!(process
    .memory_regions()
    .into_iter()
    .filter(|p| (p.Protect & mask) != 0)
    .map(|p| p.RegionSize)
    .sum::<usize>());
```

Each memory protection level has its own bit, so we can OR them all together to have a single mask. When ANDing this mask with the protection level, if any bit is set, it will be non-zero, meaning we want to keep this region.

Don't ask me why there isn't a specific bit for "write", "read", "execute", and there are only bits for combinations. I guess this way Windows forbids certain combinations.

```
2580480
```

Hey, that's close to the value shown by the Task Manager! A handfull of megabytes is a lot more manageable than 4 entire gigabytes.

## Actually running our First Scan

Okay, we have all the memory regions from which the program can read, write, or execute. Now we also can read the memory in these regions:

```rust
let regions = process
    .memory_regions()
    .into_iter()
    .filter(|p| (p.Protect & mask) != 0)
    .collect::<Vec<_>>();

println!("Scanning {} memory regions", regions.len());

regions.into_iter().for_each(|region| {
    match process.read_memory(region.BaseAddress as _, region.RegionSize) {
        Ok(memory) => todo!(),
        Err(err) => eprintln!(
            "Failed to read {} bytes at {:?}: {}",
            region.RegionSize, region.BaseAddress, err,
        ),
    }
})
```

All that's left is for us to scan for a target value. To do this, we want to iterate over all the [`slice::windows`][slicewin] of size equal to the size of our scan type.

```rust
let target: i32 = ...;
let target = target.to_ne_bytes();

// -snip-

// inside the Ok match, replacing the todo!() -- this is where the first scan happens
Ok(memory) => memory
    .windows(target.len())
    .enumerate()
    .for_each(|(offset, window)| {
        if window == target {
            println!(
                "Found exact value at [{:?}+{:x}]",
                region.BaseAddress, offset
            );
        }
    })
```

We convert the 32-bit exact target value to its memory representation as a byte array in [native byte order][tone]. This way we can compare the target bytes with the window bytes. Another option is to interpret the window bytes as an `i32` with `from_be_bytes`, but `slice::windows` gives us slices of type `&[u8]`, and `from_be_bytes` wants an `[u8; 4]` array, so it's a bit more annoying to convert.

This is enough to find the value in the process' memory!

```
Found exact value at [0x10000+aec]
Failed to read 12288 bytes at 0x13f8000: Only part of a ReadProcessMemory or WriteProcessMemory request was completed. (os error 299)
Found exact value at [0x14f0000+3188]
Found exact value at [0x14f0000+ac74]
...
Found exact value at [0x10030e000+1816]
Found exact value at [0x7ff8f7b93000+441a]
...
Found exact value at [0x7ff8fb381000+4023]
```

The tutorial starts out with health "100", which is what I scanned. Apparently, there are nearly a hundred of `100`-valued integers stored in the memory of the tutorial.

Attentive readers will notice that some values are located at an offset modulo 4. In Cheat Engine, this is known as "Fast Scan", which is enabled by default with an alignment of 4. Most of the time, values are aligned in memory, and this alignment often corresponds with the size of the type itself. For 4-byte integers, it's common that they're 4-byte aligned.

We can perform a fast scan ourselves with [`step_by`][stepby][^5]:

```rust
memory
    .windows(target.len())
    .enumerate()
    .step_by(4)
    .for_each(...)
```

As a bonus, over half the addresses are gone, so we have less results to worry about[^6].

## Next Scan

The first scan gave us way too many results. We have no way to tell which is the correct one, as they all have the same value. What we need to do is a *second* scan at the *locations we just found*. This way, we can get a second reading, and compare it against a new value. If it's the same, we're on good track, and if not, we can discard a location. Repeating this process lets us cut the hundreds of potential addresses to just a handful of them.

For example, let's say we're scanning our current health of `100` in a game. This gives us over a hundred addresses that point to the value of `100`. If we go in-game and get hit[^7] by some enemy and get our health down to, say, `99` (we have a lot of defense), we can then read the memory at the hundred memory locations we found before. If this second reading is not `99`, we know the address does not actually point to our health pool and it just happened to also contain a `100` on the first scan. This address can be removed from the list of potential addresses pointing to our health.

Let's do that:

```rust
// new vector to hold the locations, before getting into `memory.windows`' for-each
let mut locations = Vec::with_capacity(regions.len());

// -snip-

// updating the `println!("Found exact value...")` to store the location instead.
if window == target {
    locations.push(region.BaseAddress as usize + offset);
}

// -snip-

// performing a second scan on the locations the first scan found.
let target: i32 = ...;
let target = target.to_ne_bytes();
locations.retain(|addr| match process.read_memory(*addr, target.len()) {
    Ok(memory) => memory == target,
    Err(_) => false,
});

println!("Now have {} locations", locations.len());
```

We create a vector to store all the locations the first scan finds, and then retain those that match a second target value. You may have noticed that we perform a memory read, and thus a call to the Windows API, for every single address. With a hundred locations to read from, this is not a big deal, but oftentimes you will have tens of thousands of addresses. For the time being, we will not worry about this inefficiency, but we will get back to it once it matters:

```
Scanning 98 memory regions
Which exact value to scan for?: 100
Failed to read 12288 bytes at 0x13f8000: Only part of a ReadProcessMemory or WriteProcessMemory request was completed. (os error 299)
...
Found 49 locations
Which exact value to scan for next?: 99
Now have 1 locations
```

Sweet! In a real-world scenario, you will likely need to perform these additional scans a couple of times, and even then, there may be more than one value left no matter what.

For good measure, we'll wrap our `retain` in a `while` loop[^8]:

```rust
while locations.len() != 1 {
    let target: i32 = ...;
    let target = target.to_ne_bytes();
    locations.retain(...);
}
```

## Modifying memory

Now that we have very likely locations pointing to our current health in memory, all that's left is writing our new desired value to gain infinite health[^9]. Much like how we're able to read memory with `ReadProcessMemory`, we can write to it with [`WriteProcessMemory`][writemem]. Its usage is straightforward:

```rust
pub fn write_memory(&self, addr: usize, value: &[u8]) -> io::Result<usize> {
    let mut written = 0;

    // SAFETY: the input value buffer points to valid memory.
    if unsafe {
        winapi::um::memoryapi::WriteProcessMemory(
            self.handle.as_ptr(),
            addr as *mut _,
            value.as_ptr().cast(),
            value.len(),
            &mut written,
        )
    } == FALSE
    {
        Err(io::Error::last_os_error())
    } else {
        Ok(written)
    }
}
```

Similar to how writing to a file can return short, writing to a memory location could also return short. Here we mimic the API for writing files and return the number of bytes written. The documentation indicates that we could actually ignore the amount written by passing `ptr::null_mut()` as the last parameter, but it does no harm to retrieve the written count as well.

```rust
let new_value: i32 = ...;
locations
    .into_iter()
    .for_each(|addr| match process.write_memory(addr, &new_value) {
        Ok(n) => eprintln!("Written {} bytes to [{:x}]", n, addr),
        Err(e) => eprintln!("Failed to write to [{:x}]: {}", addr, e),
    });
```

And just like that:

```
Now have 1 location(s)
Enter new memory value: 1000
Failed to write to [15d8b90]: Access is denied. (os error 5)
```

…oh noes. Oh yeah. The documentation, which I totally didn't forget to read, mentions:

> The handle must have `PROCESS_VM_WRITE` and `PROCESS_VM_OPERATION` access to the process.

We currently open our process with `PROCESS_QUERY_INFORMATION` and `PROCESS_VM_READ`, which is enough for reading, but not for writing. Let's adjust `OpenProcess` to accomodate for our new requirements:

```rust
winapi::um::processthreadsapi::OpenProcess(
    winnt::PROCESS_QUERY_INFORMATION
        | winnt::PROCESS_VM_READ
        | winnt::PROCESS_VM_WRITE
        | winnt::PROCESS_VM_OPERATION,
    FALSE,
    pid,
)
```

Behold:

```
Now have 1 location(s)
Enter new memory value: 1000
Written 4 bytes to [15d8b90]
```

![Tutorial complete with memo][completion]

Isn't that active *Next* button just beautiful?

## Finale

This post somehow ended up being longer than part one, but look at what we've achieved! We completed a step of the Cheat Engine Tutorial *without using Cheat Engine*. Just pure Rust. Figuring out how a program works and reimplementing it yourself is a great way to learn what it's doing behind the scenes. And now that this code is yours, you can extend it as much as you like, without being constrained by Cheat Engine's UI. You can automate it as much as you want.

And we're not even done. The current tutorial has nine steps, and three additional graphical levels.

In the next post, we'll tackle the third step of the tutorial: Unknown initial value. This will pose a challenge, because with just 2 MiB of memory, storing all the 4-byte aligned locations would require 524288 addresses (`usize`, 8 bytes). This adds up to twice as much memory as the original program (4 MiB), but that's not our main concern, having to perform over five hundred thousand API calls is!

Remember that you can [obtain the code for this post][code] over at my GitHub. You can run `git checkout step2` after cloning the repository to get the right version of the code.

### Footnotes

[^1]: I did in fact use an online tool to spell it out for me.

[^2]: 16 GiB is good enough for my needs. I don't think I'll ever upgrade to 16 EiB.

[^3]: Every address we query should have a corresponding region, even if it's not allocated or we do not have access. This is why we can query for the memory address zero to get its corresponding region.

[^4]: Another option is to [`GetSystemInfo`][getsysinfo] to determine the `lpMinimumApplicationAddress` and `lpMaximumApplicationAddress` and only work within bounds.

[^5]: Memory regions are page-aligned, which is a large power of two. Our alignment of 4 is much lower than this, so we're guaranteed to start off at an aligned address.

[^6]: If it turns out that the value was actually misaligned, we will miss it. You will notice this if, after going through the whole process, there are no results. It could mean that either the value type is wrong, or the value type is misaligned. In the worst case, the value is not stored directly but is rather computed with something like `maximum - stored`, or XORed with some magic value, or a myriad other things.

[^7]: You could do this without getting hit, and just keep on repeating the scan for the same value over and over again. This does work, but the results are suboptimal, because there are also many other values that didn't change. Scanning for a changed value is a better option.

[^8]: You could actually just go ahead and try to modify the memory at the hundred addresses you just found, although don't be surprised if the program starts to misbehave!

[^9]: Okay, we cannot fit infinity in an `i32`. However, we can fit sufficiently large numbers. Like `1000`, which is enough to complete the tutorial.

[script-kid]: https://www.urbandictionary.com/define.php?term=script%20kiddie
[readmem]: https://docs.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-readprocessmemory
[exbibytes]: https://en.wikipedia.org/wiki/Orders_of_magnitude_(data)
[vquery]: https://docs.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualqueryex
[prdebug]: https://github.com/retep998/winapi-rs/issues/548#issuecomment-355278090
[memprot]: https://docs.microsoft.com/en-us/windows/win32/memory/memory-protection-constants
[getsysinfo]: https://docs.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getsysteminfo
[slicewin]: https://doc.rust-lang.org/stable/std/primitive.slice.html#method.windows
[tone]: https://doc.rust-lang.org/stable/std/primitive.i32.html#method.to_ne_bytes
[stepby]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#method.step_by
[writemem]: https://docs.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-writeprocessmemory
[completion]: https://user-images.githubusercontent.com/6297805/107829541-3f4f2d00-6d8a-11eb-87c4-e2f2d505afbc.png
[code]: https://github.com/lonami/memo
