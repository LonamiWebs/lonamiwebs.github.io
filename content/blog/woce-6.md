+++
title = "Writing our own Cheat Engine: Pointers"
date = 2021-03-13
updated = 2021-03-13
[taxonomies]
category = ["sw"]
tags = ["windows", "rust", "hacking"]
+++

This is part 6 on the *Writing our own Cheat Engine* series:

* [Part 1: Introduction](/blog/woce-1) (start here if you're new to the series!)
* [Part 2: Exact Value scanning](/blog/woce-2)
* [Part 3: Unknown initial value](/blog/woce-3)
* [Part 4: Floating points](/blog/woce-4)
* [Part 5: Code finder](/blog/woce-5)
* Part 6: Pointers
* [Part 7: Code Injection](/blog/woce-7)

In part 5 we wrote our very own debugger. We learnt that Cheat Engine is using hardware breakpoints to watch memory change, and how to do the same ourselves. We also learnt that hardware points are not the only way to achieve the effect of watchpoints, although they certainly are the fastest and cleanest approach.

In this post, we will be reusing some of that knowledge to find out a closely related value, the *pointer* that points to the real value[^1]. As a quick reminder, a pointer is nothing but an `usize`[^2] representing the address of another portion of memory, in this case, the actual value we will be scanning for. A pointer is a value that, well, points elsewhere. In Rust we normally use reference instead, which are safer (typed and their lifetime is tracked) than pointers, but in the end we can achieve the same with both.

Why care about pointers? It turns out that things, such as your current health in-game, are very unlikely to end up in the same memory position when you restart the game (or even change to another level, or even during gameplay). So, if you perform a scan and find that the address where your health is stored is `0x73AABABE`, you might be tempted to save it and reuse it next time you launch the game. Now you don't need to scan for it again! Alas, as soon as you restart the game, the health is now stored at `0x5AADBEEF`.

Not all hope is lost! The game must *somehow* have a way to reliably find this value, and the way it's done is with pointers. There will always be some base address that holds a pointer, and the game code knows where to find this pointer. If we are also able to find the pointer at said base address, and follow it ourselves ("dereferencing" it), we can perform the same steps the game is doing, and reliably find the health no matter how much we restart the game[^3].

## Pointers

<details open><summary>Cheat Engine Tutorial: Step 6</summary>

> In the previous step I explained how to use the Code finder to handle changing locations. But that method alone makes it difficult to find the address to set the values you want. That's why there are pointers:
>
> At the bottom you'll find 2 buttons. One will change the value, and the other changes the value AND the location of the value. For this step you don't really need to know assembler, but it helps a lot if you do.
>
> First find the address of the value. When you've found it use the function to find out what accesses this address.
>
> Change the value again, and a item will show in the list. Double click that item. (or select and click on more info) and a new window will open with detailed information on what happened when the instruction ran.
>
> If the assembler instruction doesn't have anything between a '[' and ']' then use another item in the list. If it does it will say what it think will be the value of the pointer you need.
>
> Go back to the main cheat engine window (you can keep this extra info window open if you want, but if you close it, remember what is between the \[ and \]) and do a 4 byte scan in hexadecimal for the value the extra info told you. When done scanning it may return 1 or a few hundred addresses. Most of the time the address you need will be the smallest one. Now click on manually add and select the pointer checkbox.
>
> The window will change and allow you to type in the address of a pointer and a offset. Fill in as address the address you just found. If the assembler instruction has a calculation (e.g: [esi+12]) at the end then type the value in that's at the end. else leave it 0. If it was a more complicated instruction look at the calculation.
>
> Example of a more complicated instruction:
>
> [EAX*2+EDX+00000310] eax=4C and edx=00801234.
>
> In this case EDX would be the value the pointer has, and EAX\*2+00000310 the offset, so the offset you'd fill in would be 2\*4C+00000310=3A8.  (this is all in hex, use calc.exe from windows in scientific mode to calculate).
>
> Back to the tutorial, click OK and the address will be added, If all went right the address will show P->xxxxxxx, with xxxxxxx being the address of the value you found. If thats not right, you've done something wrong. Now, change the value using the pointer you added in 5000 and freeze it. Then click Change pointer, and if all went right the next button will become visible.
>
> *extra*: And you could also use the pointer scanner to find the pointer to this address.

</details>

## On-access watchpoints

Last time we managed to learn how hardware breakpoints were being set by observing Cheat Engine's behaviour. I think it's now time to handle this properly instead. We'll check out the [CPU Registers x86 page on OSDev][dbg-reg] to learn about it:

* DR0, DR1, DR2 and DR3 can hold a memory address each. This address will be used by the breakpoint.
* DR4 is actually an [obsolete synonym][dr4] for DR6.
* DR5 is another obsolete synonym, this time for DR7.
* DR6 is debug status. The four lowest bits indicate which breakpoint was hit, and the four highest bits contain additional information. We should make sure to clear this ourselves when a breakpoint is hit.
* DR7 is debug control, which we need to study more carefully.

Each debug register DR0 through DR3 has two corresponding bits in DR7, starting from the lowest-order bit, to indicate whether the corresponding register is a **L**ocal or **G**lobal breakpoint. So it looks like this:

```
  Meaning: [ .. .. | G3 | L3 | G2 | L2 | G1 | L1 | G0 | L0 ]
Bit-index:   31-08 | 07 | 06 | 05 | 04 | 03 | 02 | 01 | 00
```

Cheat Engine was using local breakpoints, because the zeroth bit was set. Probably because we don't want these breakpoints to infect other programs! Because we were using only one breakpoint, only the lowermost bit was being set. The local 1st, 2nd and 3rd bits were unset.

Now, each debug register DR0 through DR4 has four additional bits in DR7, two for the **C**ondition and another two for the **S**ize:

```
  Meaning: [   S3  |   C3  |   S2  |   C2  |   S1  |   C1  |   S0  |   C0  | .. .. ]
Bit-index:   31 30 | 29 28 | 27 26 | 25 24 | 23 22 | 21 20 | 19 18 | 17 16 | 15-00
```

The two bits of the condition mean the following:

* `00` execution breakpoint.
* `01` write watchpoint.
* `11` read/write watchpoint.
* `10` unsupported I/O read/write.

When we were using Cheat Engine to add write watchpoints, the bits 17 and 16 were indeed set to `01`, and the bits 19 and 18 were set to `11`. Hm, but *11<sub>2</sub>&nbsp;=&nbsp;3<sub>10</sub>*&nbsp;, and yet, we were watching writes to 4 bytes. So what's up with this? Is there a different mapping for the size which isn't documented at the time of writing? Seems we need to learn from Cheat Engine's behaviour one more time.

For reference, this is what DR7 looked like when we added a single write watchpoint:

```
hex: 000d_0001
bin: 00000000_00001101_00000000_00000001
```

And this is the code I will be using to check the breakpoints of different sizes:

```
thread::enum_threads(pid)
    .unwrap()
    .into_iter()
    .for_each(|tid| {
        let thread = thread::Thread::open(tid).unwrap();
        let ctx = thread.get_context().unwrap();
        eprintln!("hex: {:08x}", ctx.Dr7);
        eprintln!("bin: {:032b}", ctx.Dr7);
    });
```

Let's compare this to watchpoints for sizes 1, 2, 4 and 8 bytes:

```
1 byte
hex: 0001_0401
bin: 00000000_00000001_00000100_00000001

2 bytes
hex: 0005_0401
bin: 00000000_00000101_00000100_00000001

4 bytes
hex: 000d_0401
bin: 00000000_00001101_00000100_00000001

8 bytes
hex: 0009_0401
bin: 00000000_00001001_00000100_00000001
                            ^ wut?
```

I have no idea what's up with that stray tenth bit. Its use does not seem documented, and things worked fine without it, so we'll ignore it. The lowest bit is set to indicate we're using DR0, bits 17 and 16 represent the write watchpoint, and the size seems to be as follows:

* `00` for a single byte.
* `01` for two bytes (a "word").
* `11` for four bytes (a "double word").
* `10` for eight bytes (a "quadruple word").

Doesn't make much sense if you ask me, but we'll roll with it. Just to confirm, this is what the "on-access" breakpoint looks like according to Cheat Engine:

```
hex: 000f_0401
bin: 00000000_00001111_00000100_00000001
```

So it all checks out! The bit pattern is `11` for read/write (technically, a write is also an access). Let's implement this!

## Proper breakpoint handling

The first thing we need to do is represent the possible breakpoint conditions:

```rust
#[repr(u8)]
pub enum Condition {
    Execute = 0b00,
    Write = 0b01,
    Access = 0b11,
}
```

And also the legal breakpoint sizes:

```rust
#[repr(u8)]
pub enum Size {
    Byte = 0b00,
    Word = 0b01,
    DoubleWord = 0b11,
    QuadWord = 0b10,
}
```

We are using `#[repr(u8)]` so that we can convert a given variant into the corresponding bit pattern. With the right types defined in order to set a breakpoint, we can start implementing the method that will set them (inside `impl Thread`):

```rust
pub fn add_breakpoint(&self, addr: usize, cond: Condition, size: Size) -> io::Result<Breakpoint> {
    let mut context = self.get_context()?;
    todo!()
}
```

First, let's try finding an "open spot" where we could set our breakpoint. We will "slide" a the `0b11` bitmask over the lowest eight bits, and if and only if both the local and global bits are unset, then we're free to set a breakpoint at this index[^4]:

```rust
let index = (0..4)
    .find_map(|i| ((context.Dr7 & (0b11 << (i * 2))) == 0).then(|| i))
    .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no debug register available"))?;
```

Once an `index` is found, we can set the address we want to watch in the corresponding register and update the debug control bits:

```rust
let addr = addr as u64;
match index {
    0 => context.Dr0 = addr,
    1 => context.Dr1 = addr,
    2 => context.Dr2 = addr,
    3 => context.Dr3 = addr,
    _ => unreachable!(),
}

let clear_mask = !((0b1111 << (16 + index * 4)) | (0b11 << (index * 2)));
context.Dr7 &= clear_mask;

context.Dr7 |= 1 << (index * 2);

let sc = (((size as u8) << 2) | (cond as u8)) as u64;
context.Dr7 |= sc << (16 + index * 4);

self.set_context(&context)?;
Ok(Breakpoint {
    thread: self,
    clear_mask,
})
```

Note that we're first creating a "clear mask". We switch on all the bits that we may use for this breakpoint, and then negate. Effectively, `Dr7 & clear_mask` will make sure we don't leave any bit high on accident. We apply the mask before OR-ing the rest of bits to also clear any potential garbage on the size and condition bits. Next, we set the bit to enable the new local breakpoint, and also store the size and condition bits at the right location.

With the context updated, we can set it back and return the `Breakpoint`. It stores the `thread` and the `clear_mask` so that it can clean up on `Drop`. We are technically relying on `Drop` to run behaviour here, but the cleanup is done on a best-effort basis. If the user intentionally forgets the `Breakpoint`, maybe they want the `Breakpoint` to forever be set.

This logic is begging for a testcase though; I'll split it into a new `Breakpoint::update_dbg_control` method and test that out:

```rust

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brk_add_one() {
        // DR7 starts with garbage which should be respected.
        let (clear_mask, dr, dr7) =
            Breakpoint::update_dbg_control(0x1700, Condition::Write, Size::DoubleWord).unwrap();

        assert_eq!(clear_mask, 0xffff_ffff_fff0_fffc);
        assert_eq!(dr, DebugRegister::Dr0);
        assert_eq!(dr7, 0x0000_0000_000d_1701);
    }

    #[test]
    fn brk_add_two() {
        let (clear_mask, dr, dr7) = Breakpoint::update_dbg_control(
            0x0000_0000_000d_0001,
            Condition::Write,
            Size::DoubleWord,
        )
        .unwrap();

        assert_eq!(clear_mask, 0xffff_ffff_ff0f_fff3);
        assert_eq!(dr, DebugRegister::Dr1);
        assert_eq!(dr7, 0x0000_0000_00dd_0005);
    }

    #[test]
    fn brk_try_add_when_max() {
        assert!(Breakpoint::update_dbg_control(
            0x0000_0000_dddd_0055,
            Condition::Write,
            Size::DoubleWord
        )
        .is_none());
    }
}
```

```
running 3 tests
test thread::tests::brk_add_one ... ok
test thread::tests::brk_add_two ... ok
test thread::tests::brk_try_add_when_max ... ok
```

Very good! With proper breakpoint handling usable, we can continue.

## Inferring the pointer value

After scanning memory for the location we're looking for (say, our current health), we then add an access watchpoint, and wait for an exception to occur. As a reminder, here's the page with the [Debugging Events][dbg-events]:

```rust
let addr = ...;
let mut threads = ...;

let _watchpoints = threads
    .iter_mut()
    .map(|thread| {
        thread
            .add_breakpoint(addr, thread::Condition::Access, thread::Size::DoubleWord)
            .unwrap()
    })
    .collect::<Vec<_>>();

loop {
    let event = debugger.wait_event(None).unwrap();
    if event.dwDebugEventCode == winapi::um::minwinbase::EXCEPTION_DEBUG_EVENT {
        let exc = unsafe { event.u.Exception() };
        if exc.ExceptionRecord.ExceptionCode == winapi::um::minwinbase::EXCEPTION_SINGLE_STEP {
            todo!();
        }
    }
    debugger.cont(event, true).unwrap();
}
```

Now, inside the `todo!()` we will want to do a few things, namely printing out the instructions "around this location" and dumping the entire thread context on screen. To print the instructions, we need to import `iced_x86` again, iterate over all memory regions to find the region where the exception happened, read the corresponding bytes, decode the instructions, and when we find the one with a corresponding instruction pointer, print "around it":

```rust
use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, NasmFormatter};

let addr = exc.ExceptionRecord.ExceptionAddress as usize;
let region = process
    .memory_regions()
    .into_iter()
    .find(|region| {
        let base = region.BaseAddress as usize;
        base <= addr && addr < base + region.RegionSize
    })
    .unwrap();

let bytes = process
    .read_memory(region.BaseAddress as usize, region.RegionSize)
    .unwrap();

let mut decoder = Decoder::new(64, &bytes, DecoderOptions::NONE);
decoder.set_ip(region.BaseAddress as _);

let mut formatter = NasmFormatter::new();
let mut output = String::new();

let instructions = decoder.into_iter().collect::<Vec<_>>();
for (i, ins) in instructions.iter().enumerate() {
    if ins.next_ip() as usize == addr {
        let low = i.saturating_sub(5);
        let high = (i + 5).min(instructions.len());
        for j in low..high {
            let ins = &instructions[j];
            print!("{} {:016X} ", if j == i { ">>>" } else { "   " }, ins.ip());
            let k = (ins.ip() - region.BaseAddress as usize as u64) as usize;
            let instr_bytes = &bytes[k..k + ins.len()];
            for b in instr_bytes.iter() {
                print!("{:02X}", b);
            }
            if instr_bytes.len() < 10 {
                for _ in 0..10usize.saturating_sub(instr_bytes.len()) {
                    print!("  ");
                }
            }

            output.clear();
            formatter.format(ins, &mut output);
            println!(" {}", output);
        }
        break;
    }
}
debugger.cont(event, true).unwrap();
break;
```

The result is pretty fancy:

```
    000000010002CAAC 48894DF0             mov [rbp-10h],rcx
    000000010002CAB0 488955F8             mov [rbp-8],rdx
    000000010002CAB4 48C745D800000000     mov qword [rbp-28h],0
    000000010002CABC 90                   nop
    000000010002CABD 488B050CA02D00       mov rax,[rel 100306AD0h]
>>> 000000010002CAC4 8B00                 mov eax,[rax]
    000000010002CAC6 8945EC               mov [rbp-14h],eax
    000000010002CAC9 B9E8030000           mov ecx,3E8h
    000000010002CACE E88D2FFEFF           call 000000010000FA60h
    000000010002CAD3 8945E8               mov [rbp-18h],eax
```

Cool! So `rax` is holding an address, meaning it's a pointer, and the value it reads (dereferences) is stored back into `eax` (because it does not need `rax` anymore). Alas, the current thread context has the register state *after* the instruction was executed, and `rax` no longer contains the address at this point. However, notice how the previous instruction writes a fixed value to `rax`, and then that value is used to access memory, like so:

```rust
let eax = memory[memory[0x100306AD0]];
```

The value at `memory[0x100306AD0]` *is* the pointer! No offsets are used, because nothing is added to the pointer after it's read. This means that, if we simply scan for the address we were looking for, we should find out where the pointer is stored:

```rust
let addr = ...;
let scan = process.scan_regions(&regions, Scan::Exact(addr as u64));

scan.into_iter().for_each(|region| {
    region.locations.iter().for_each(|ptr_addr| {
        println!("[{:x}] = {:x}", ptr_addr, addr);
    });
});
```

And just like that:

```
[100306ad0] = 15de9f0
```

Notice how the pointer address found matches with the offset used by the instructions:

```
    000000010002CABD 488B050CA02D00       mov rax,[rel 100306AD0h]
           this is the same as the value we just found ^^^^^^^^^^
```

Very interesting indeed. We were actually very lucky to have only found a single memory location containing the pointer value, `0x15de9f0`. Cheat Engine somehow knows that this value is always stored at `0x100306ad0` (or rather, at `Tutorial-x86_64.exe+306AD0`), because the address shows green. How does it do this?

## Base addresses

Remember back in [part 2](/blog/woce-2) when we introduced the memory regions? They're making a comeback! A memory region contains both the current memory protection option *and* the protection level when the region was created. If we try printing out the protection levels for both the memory region containing the value, and the memory region containing the pointer, this is what we get (the addresses differ from the ones previously because I restarted the tutorial):

```
Region holding the value:
    BaseAddress: 0xb0000
    AllocationBase: 0xb0000
    AllocationProtect: 0x4
    RegionSize: 1007616
    State: 4096
    Protect: 4
    Type: 0x20000

Region holding the pointer:
    BaseAddress: 0x100304000
    AllocationBase: 0x100000000
    AllocationProtect: 0x80
    RegionSize: 28672
    State: 4096
    Protect: 4
    Type: 0x1000000
```

Interesting! According to the [`MEMORY_BASIC_INFORMATION` page][meminfo], the type for the first region is `MEM_PRIVATE`, and the type for the second region is `MEM_IMAGE` which:

> Indicates that the memory pages within the region are mapped into the view of an image section.

The protection also changes from `PAGE_EXECUTE_WRITECOPY` to simply `PAGE_READWRITE`, but I don't think it's relevant. Neither the type seems to be much more relevant. In [part 2](/blog/woce-2) we also mentioned the concept of "base address", but decided against using it, because starting to look for regions at address zero seemed to work fine. However, it would make sense that fixed "addresses" start at some known "base". Let's try getting the [base address for all loaded modules][baseaddr]. Currently, we only get the address for the base module, in order to retrieve its name, but now we need them all:

```rust
pub fn enum_modules(&self) -> io::Result<Vec<winapi::shared::minwindef::HMODULE>> {
    let mut size = 0;
    if unsafe {
        winapi::um::psapi::EnumProcessModules(
            self.handle.as_ptr(),
            ptr::null_mut(),
            0,
            &mut size,
        )
    } == FALSE
    {
        return Err(io::Error::last_os_error());
    }

    let mut modules = Vec::with_capacity(size as usize / mem::size_of::<HMODULE>());
    if unsafe {
        winapi::um::psapi::EnumProcessModules(
            self.handle.as_ptr(),
            modules.as_mut_ptr(),
            (modules.capacity() * mem::size_of::<HMODULE>()) as u32,
            &mut size,
        )
    } == FALSE
    {
        return Err(io::Error::last_os_error());
    }

    unsafe {
        modules.set_len(size as usize / mem::size_of::<HMODULE>());
    }

    Ok(modules)
}
```

The first call is used to retrieve the correct `size`, then we allocate just enough, and make the second call. The returned type are pretty much memory addresses, so let's see if we can find regions that contain them:

```rust
let mut bases = 0;
let modules = process.enum_modules().unwrap();
let regions = process.memory_regions();
regions.iter().for_each(|region| {
    if modules.iter().any(|module| {
        let base = region.AllocationBase as usize;
        let addr = *module as usize;
        base <= addr && addr < base + region.RegionSize
    }) {
        bases += 1;
    }
});

println!(
    "{}/{} regions have a module address within them",
    bases,
    regions.len()
);
```

```
41/353 regions have a module address within them
```

Exciting stuff! It appears `base == addr` also does the trick[^5], so now we could build a `bases: HashSet<usize>` and simply check if `bases.contains(&region.AllocationBase as usize)` to determine whether `region` is a base address or not[^6]. So there we have it! The address holding the pointer value does fall within one of these "base regions". You can also get the name from one of these module addresses, and print it in the same way as Cheat Engine does it (such as `Tutorial-x86_64.exe+306AD0`).

## Finale

So, there's no "automated" solution to all of this? That's the end? Well, yes, once you have a pointer you can dereference it once and then write to the given address to complete the tutorial step! I can understand how this would feel a bit underwhelming, but in all fairness, we were required to pretty-print assembly to guess what pointer address we could potentially need to look for. There is an [stupidly large amount of instructions][isa], and I'm sure a lot of them can access memory, so automating that would be rough. We were lucky that the instructions right before the one that hit the breakpoint were changing the memory address, but you could imagine this value coming from somewhere completely different. It could also be using a myriad of different techniques to apply the offset. I would argue manual intervention is a must here[^7].

We have learnt how to pretty-print instructions, and had a very gentle introduction to figuring out what we may need to look for. The code to retrieve the loaded modules, and their corresponding regions, will come in handy later on. Having access to this information lets us know when to stop looking for additional pointers. As soon as a pointer is found within a memory region corresponding to a base module, we're done! Also, I know the title doesn't really much the contents of this entry (sorry about that), but I'm just following the convention of calling it whatever the Cheat Engine tutorial calls them.

The [code for this post][code] is available over at my GitHub. You can run `git checkout step6` after cloning the repository to get the right version of the code, although you will have to `checkout` to individual commits if you want to review, for example, how the instructions were printed out. Only the code necessary to complete the step is included at the `step6` tag.

In the [next post](/blog/woce-7), we'll tackle the seventh step of the tutorial: Code Injection. This will be pretty similar to part 5, but instead of writing out a simple NOP instruction, we will have to get a bit more creative.

### Footnotes

[^1]: This will only be a gentle introduction to pointers. Part 8 of this series will have to rely on more advanced techniques.

[^2]: Kind of. The size of a pointer isn't necessarily the size as `usize`, although `usize` is guaranteed to be able of representing every possible address. For our purposes, we can assume a pointer is as big as `usize`.

[^3]: Game updates are likely to pull more code and shuffle stuff around. This is unfortunately a difficult problem to solve. But storing a pointer which is usable across restarts for as long as the game doesn't update is still a pretty darn big improvement over having to constantly scan for the locations we care about. Although if you're smart enough to look for certain unique patterns, even if the code is changed, finding those patterns will give you the new updated address, so it's not *impossible*.

[^4]: `bool::then` is a pretty recent addition at the time of writing (1.50.0), so make sure you `rustup update` if it's erroring out!

[^5]: I wasn't sure if there would be some metadata before the module base address but within the region, so I went with the range check. What *is* important however is using `AllocationBase`, not `BaseAddress`. They're different, and this did bite me.

[^6]: As usual, I have no idea if this is how Cheat Engine is doing it, but it seems reasonable.

[^6]: But nothing's stopping you from implementing some heuristics to get the job done for you. If you run some algorithm in your head to find what the pointer value could be, you can program it in Rust as well, although I don't think it's worth the effort.

[dbg-reg]: https://wiki.osdev.org/CPU_Registers_x86#Debug_Registers
[dr4]: https://en.wikipedia.org/wiki/X86_debug_register
[dbg-events]: https://docs.microsoft.com/en-us/windows/win32/debug/debugging-events
[meminfo]: https://docs.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-memory_basic_information
[baseaddr]: https://stackoverflow.com/a/26573045/4759433
[isa]: https://www.intel.com/content/www/us/en/architecture-and-technology/64-ia-32-architectures-software-developer-vol-2a-manual.html
[code]: https://github.com/lonami/memo
