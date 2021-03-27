+++
title = "Writing our own Cheat Engine: Code Injection"
date = 2021-05-08
updated = 2021-05-08
[taxonomies]
category = ["sw"]
tags = ["windows", "rust", "hacking"]
+++

This is part 7 on the *Writing our own Cheat Engine* series:

* [Part 1: Introduction](/blog/woce-1) (start here if you're new to the series!)
* [Part 2: Exact Value scanning](/blog/woce-2)
* [Part 3: Unknown initial value](/blog/woce-3)
* [Part 4: Floating points](/blog/woce-4)
* [Part 5: Code finder](/blog/woce-5)
* [Part 6: Pointers](/blog/woce-6)
* Part 7: Code Injection

In part 6 we ended up spending most of the time in upgrading our breakpoint support to have a proper implementation, rather than using some hardcoded constants. We then made use of the new and improved breakpoint support to find what code accessed an specific memory address our very own debugger. To complete the tutorial, we read and understood the surrounding assembly around the code accessing our address and figured out what pointer to look for. In the end, we were left with a base address that we can rely on and follow to reach the target memory address, without having to scan for it every time.

In this post, we will take a look at the different techniques Cheat Engine uses to patch instructions with as many other instructions as we need.

## Code Injection

<details open><summary>Cheat Engine Tutorial: Step 7</summary>

> Code injection is a technique where you inject a piece of code into the target process, and then reroute the execution of code to go through your own written code.
>
> In this tutorial you'll have a health value and a button that will decrease your health by 1 each time you click it. Your task is to use code injection to make the button increase your health by 2 each time it is clicked.
>
> Start with finding the address and then find what writes to it. Then when you've found the code that decreases it browse to that address in the disassembler, and open the auto assembler window (ctrl+a). There click on template and then code injection, and give it the address that decreases health (if it isn't already filled in correctly). That will generate a basic auto assembler injection framework you can use for your code.
>
> Notice the alloc, that will allocate a block of memory for your code cave, in the past, in the pre windows 2000 systems, people had to find code caves in the memory (regions of memory unused by the game), but that's luckily a thing of the past since windows 2000, and will these days cause errors when trying to be used, due to SP2 of XP and the NX bit of new CPU's
>
> Also notice the line `newmem:` and `originalcode:` and the text "Place your code here". As you guessed it, write your code here that will increase the health with 2. An usefull assembler instruction in this case is the "ADD instruction". Here are a few examples:
>
> * "ADD [00901234],9" to increase the address at 00901234 with 9
> * "ADD [ESP+4],9" to increase the address pointed to by ESP+4 with 9
>
> In this case, you'll have to use the same thing between the brackets as the original code has that decreases your health
>
> Notice: It is recommended to delete the line that decreases your health from the original code section, else you'll have to increase your health with 3 (you increase with 3, the original code decreases with 1, so the end result is increase with 2), which might become confusing. But it's all up to you and your programming.
>
> Notice 2: In some games the original code can exist out of multiple instructions, and sometimes, not always, it might happen that a code at another place jumps into your jump instruction end will then cause unknown behavior. If that happens, you should usually look near that instruction and see the jumps and fix it, or perhaps even choose to use a different address to do the code injection from. As long as you're able to figure out the address to change from inside your injected code.

</details>

## Injection techniques

The Instruction Set Architecture (ISA) a typical desktop computer is able to interpret uses a variable-length encoding for the instructions (do correct me if this is phrased incorrectly; it's not my area of expertise). That means we can't go and blindly replace a instruction with the code we need. We need to be careful, and still hope that no code dynamically jumps to this very specific location. Otherwise we may end up executing [Unintended Instructions][unintended]!

The way Cheat Engine gets around this is by replacing the instruction with a jump. After the offending code is found, you can use a "template" that prompts "On what address do you want the jump?". After accepting the "code inject template", a window with the following code shows:

```asm
alloc(newmem,2048,"Tutorial-x86_64.exe"+2D4F7)
label(returnhere)
label(originalcode)
label(exit)

newmem: //this is allocated memory, you have read,write,execute access
//place your code here

originalcode:
sub dword ptr [rsi+000007E0],01

exit:
jmp returnhere

"Tutorial-x86_64.exe"+2D4F7:
jmp newmem
nop 2
returnhere:
```

It seems Cheat Engine has its own mini-language that extends assembly using Intel-syntax. It has `directive(arguments)` which do… well, stuff.

`alloc(label, size, address)` seems to allocate `size` bytes at some address and assign `label` to it. `address` is where the jump to the newly-allocated memory will be inserted.

`label(label)` seems to be used to define a label. Unlike your usual assembler, it appears we need to define the labels beforehand.

A label may also be an address directly, in this case, `"Tutorial-x86_64.exe"+2D4F7`. Cheat Engine will overwrite code from this address onwards.

Executing Cheat Engine's assembler will greet you with the following message, provided everything went okay:

> **Information**
>
> The code injection was successfull\
> newmem=FFFF0000\
> Go to FFFF0000?
>
> <kbd>Yes</kbd> <kbd>No</kbd>

If we navigate to the address, we find the following:

```
...
FFFEFFFE -                       - ??
FFFEFFFF -                       - ??
FFFF0000 - 83 AE E0070000 01     - sub dword ptr [rsi+000007E0],01
FFFF0007 - E9 F2D40300           - jmp Tutorial-x86_64.exe+2D4FE
FFFF000C - 00 00                 - add [rax],al
...
FFFF0FFF - 00 00                 - add [rax],al
...
FFFF1001 -                       - ??
...
```

So, before this address we don't know what's in there. At the address, our newly inserted code is present, and after the code, a lot of zero values (which happen to be interpreted as `add [rax], al`). After the allocated region (in our case, 2048 bytes), more unknown memory follows.

The old code was replaced with the jump:

```
Tutorial-x86_64.exe+2D4F7 - E9 042BFCFF           - jmp FFFF0000
Tutorial-x86_64.exe+2D4FC - 66 90                 - nop 2
```

Note how the `sub` instruction (`83 AE E0070000 01`, 7 bytes) was replaced with both a `jmp` (`E9 042BFCFF`, 5 bytes) and a `nop` (`66 90`, 2 bytes), both occupying 7 bytes. Because the size was respected, any old jumps will still fall in the same locations. But we were lucky to be working with 7 whole bytes to ourselves. What happens if we try to do the same on, say, a `nop` which is only 1 byte long?

```asm
alloc(newmem,2048,"Tutorial-x86_64.exe"+2D4F0)
newmem:
nop
mov ebx,[rsi+000007E0]
jmp returnhere

"Tutorial-x86_64.exe"+2D4F0:
jmp newmem
nop 2
returnhere:
```

Interesting! A single byte is obviously not enough, so Cheat Engine goes ahead and replaces *two* instructions with the jump, even though we only intended to replace one. Note the old code at `newmem`, it contains the `nop` and the next instruction (this was just before the code we are meant to replace, so I picked it as the example).

Cheat Engine is obviously careful to both pick as many instructions as it needs to fit a `jmp`, and the template pads the `jmp` with as many `nop` bytes as it needs to respect the old size.

If you attempt to assemble a longer instruction to replace a smaller one inline (as opposed to use the assembler with templates), Cheat Engine will warn you:

> **Confirmation**
>
> The generated code is 6 byte(s) long, but the selected opcode is 1 byte(s) long! Do you want to replace the incomplete opcode(s) with NOP's?
>
> <kbd>Yes</kbd> <kbd>No</kbd> <kbd>Cancel</kbd>

Selecting "No" will leave the incomplete bytes as they were before (in the case you replace a long instruction with a short one), which is very likely to leave garbage instructions behind and mess up with even more instructions.

## Allocating remote memory

When we initialize a new `Vec` via `Vec::with_capacity(2048)`, Rust will allocate enough space for 2048 items in a memory region that will belong to us. But we need this memory to belong to a different process, so that the remote process is the one with full Read, Write and eXecute access.

There's quite a few ways to allocate memory: [`GlobalAlloc`][global-alloc], [`LocalAlloc`][local-alloc], [`HeapAlloc`][heap-alloc], [`VirtualAlloc`][virtual-alloc]… just to name a few! A process may even embed its own allocator which works on top of any of these. Each of these functions has its own purpose, with different tradeoffs, but the [comparison on allocation methods][allocs] notes:

> Starting with 32-bit Windows, `GlobalAlloc` and `LocalAlloc` are implemented as wrapper functions that call `HeapAlloc` using a handle to the process's default heap.

Cool! That's two down. `CoTaskMemAlloc` seems to be useful in COM-based applications, which we don't care about, and `VirtualAlloc`:

> \[…\] allows you to specify additional options for memory allocation. However, its allocations use a page granularity, so using `VirtualAlloc` can result in higher memory usage.

…which we don't care about, either. Since `HeapAlloc` requires "A handle to the heap from which the memory will be allocated", and as far as I can tell, there is no easy way to do this for a different process, we'll turn our attention back to `VirtualAlloc`. The [documentation][virtual-alloc] reads:

> To allocate memory in the address space of another process, use the [`VirtualAllocEx`] function.

There's our function! But before we can use it, we should figure out how the memory allocated by Cheat Engine looks like. I'll be using this code:

```rust
let before = process.memory_regions();
std::thread::sleep(std::time::Duration::from_secs(10));
let after = process.memory_regions();

before.iter().for_each(|pre| {
    if let Some(post) = after.iter().find(|post| post.BaseAddress == pre.BaseAddress) {
        if post.RegionSize != pre.RegionSize {
            println!("region {:?} size changed: {:x} -> {:x}", pre.BaseAddress, pre.RegionSize, post.RegionSize);
        }
        if post.Protect != pre.Protect {
            println!("region {:?} prot changed: {:x} -> {:x}", pre.BaseAddress, pre.Protect, post.Protect);
        }
    } else {
        println!("region {:?} lost", pre.BaseAddress);
    }
});

after.iter().for_each(|post| {
    if !before.iter().any(|pre| pre.BaseAddress == post.BaseAddress) {
        println!("region {:?} came to life (size {:x}, prot {:x})", post.BaseAddress, post.RegionSize, post.Protect);
    }
});
```

The results:

```
region 0x7ffe3000 size changed: 8001d000 -> 8000d000
region 0xffff0000 came to life (size 1000, prot 40)
region 0xffff1000 came to life (size f000, prot 1)
```

So far, so good. This matches the address Cheat Engine was telling us about. It appears region 0x7ffe3000 was split to accomodate for region 0xffff0000, and the remaining had to become region 0xffff1000. The protection level for the region we care about is 40, which, according to the [documentation][memprot] is `PAGE_EXECUTE_READWRITE`. It "Enables execute, read-only, or read/write access to the committed region of pages". Let's implement that in `Process`:

```rust
pub fn alloc(&self, addr: usize, size: usize) -> io::Result<usize> {
    let res = unsafe {
        winapi::um::memoryapi::VirtualAllocEx(
            self.handle.as_ptr(),
            addr as _,
            size,
            winnt::MEM_COMMIT | winnt::MEM_RESERVE,
            winnt::PAGE_EXECUTE_READWRITE,
        )
    };
    if res == ptr::null_mut() {
        Err(io::Error::last_os_error())
    } else {
        Ok(res as _)
    }
}

pub fn dealloc(&self, addr: usize) -> io::Result<()> {
    if unsafe {
        winapi::um::memoryapi::VirtualFreeEx(
            self.handle.as_ptr(),
            addr as _,
            0,
            winnt::MEM_RELEASE,
        )
    } == FALSE
    {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
```

`VirtualAllocEx` will also zero-initialize the remote memory, although we don't care much about that. To us, all the memory is initialized, because we work through `ReadProcessMemory` which is the one responsible for filling our buffers. The only fun remark is that we also saw zero-bytes when we did the process with Cheat Engine, and not random garbage, so that may be an indicator that we're on the right track.

We also provide `dealloc`, so that the user can free memory if they want to. Otherwise, they're causing a memory leak in a remote process.

## Finding the right spot

Before we go and allocate memory, we need to determine *where* it should be allocated. Remember the `jmp` instruction Cheat Engine added?:

```
Tutorial-x86_64.exe+2D4F7 - E9 042BFCFF           - jmp FFFF0000
```

It's 5 bytes long, and the "address" is 4 bytes long. However, memory addresses are 8 bytes long! And also, the argument (`042BFCFF`) to the jump (`E9`) is backwards. Our machines are little endian, so the actual value is `FFFC2B04` instead. I wonder what happens if…

```python
>>> hex(0xFFFC2B04 + 0x2D4F7 + 5)
'0xffff0000'
```

Aha! So the argument to the jump location is actually encoded *relative* to the current instruction pointer *after* reading the instruction (that's the plus five). In this case, all we need to do is find a memory region which is not yet reserved and is close enough to the offending instruction, so that we can make sure the relative offset will fit in 4 bytes:

```rust
let regions = process
    .memory_regions()
    .into_iter()
    .filter(|p| (p.State & winnt::MEM_FREE) != 0)
    .collect::<Vec<_>>();

println!("{} regions free", regions.len());
```

```
68 regions free
```

Sure enough, there's still free regions available to us. Because `memory_regions` is sorted by `BaseAddress`, we can look for the first free region after the address we want to patch:

```rust
let region = process
    .memory_regions()
    .into_iter()
    .find(|p| (p.State & winnt::MEM_FREE) != 0 && p.BaseAddress as usize > addr)
    .unwrap();

println!("Found free region at {:?}", region.BaseAddress);
```

```
Do you want to simply inject NOPs replacing the old code at 10002d4fe (y/n)?: n
Found free region at 0x100321000
```

There we go! 0x2f3b02 bytes away of 0x10002d4fe, we have a free memory region at 0x100321000 where we can allocate memory to. Alas, trying to allocate memory here fails:

```rust
Os { code: 487, kind: Other, message: "Attempt to access invalid address." }
```

Well, to be fair, that's not the region Cheat Engine is finding. Here's what the memory looks like around the region Cheat Engine does use *before* injecting the code:

```
Region:
    BaseAddress: 0x7ffe6000
    AllocationBase: 0x0
    AllocationProtect: 0
    RegionSize: 8001a000
    State: 10000
    Protect: 1
    Type: 0
Region:
    BaseAddress: 0x100000000
    AllocationBase: 0x100000000
    AllocationProtect: 80
    RegionSize: 1000
    State: 1000
    Protect: 2
    Type: 1000000
```

And here is the after:

```
Region:
    BaseAddress: 0x7ffe6000
    AllocationBase: 0x0
    AllocationProtect: 0
    RegionSize: 8000a000
    State: 10000
    Protect: 1
    Type: 0
Region:
    BaseAddress: 0xffff0000
    AllocationBase: 0xffff0000
    AllocationProtect: 40
    RegionSize: 1000
    State: 1000
    Protect: 40
    Type: 20000
Region:
    BaseAddress: 0xffff1000
    AllocationBase: 0x0
    AllocationProtect: 0
    RegionSize: f000
    State: 10000
    Protect: 1
    Type: 0
Region:
    BaseAddress: 0x100000000
    AllocationBase: 0x100000000
    AllocationProtect: 80
    RegionSize: 1000
    State: 1000
    Protect: 2
    Type: 1000000
```

Notice how the region it picked was 0x7ffe_6000, not 0x1_0000_0000. The offending instruction is at 0x1_0002_d4fe. So the jumps can go backwards just fine. But this doesn't really explain why the allocation at 0x1_0032_1000 failed, because it has the same state (`MEM_FREE`) and protection level (`PAGE_NOACCESS`) as the page at 0x7ffe_6000. I can't really explain why this is the case, but I can change the code to pick a free memory region before and not after the offending instruction:

```rust
let region = process
    .memory_regions()
    .into_iter()
    .rev() // <- new                                               flipped v
    .find(|p| (p.State & winnt::MEM_FREE) != 0 && (p.BaseAddress as usize) < addr)
    .unwrap();
```

```
Found free region at 0x7ffe6000
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Os { code: 487, kind: Other, message: "Attempt to access invalid address." }', src\main.rs:151:74
```

Perhaps the two regions aren't so different after all? At least we're picking the same region as Cheat Engine now. But why is the allocation failing? I'll be honest, I have no idea. We do have the required `PROCESS_VM_OPERATION` permission. I do not think the error is caused by enclaves (and I don't even know what those are):

> If the address in within an enclave that you initialized, then the allocation operation fails with the `ERROR_INVALID_ADDRESS` error.

It also does not seem to be an issue with reserve and commit:

> Attempting to commit a specific address range by specifying `MEM_COMMIT` without `MEM_RESERVE` and a non-`NULL` `lpAddress` fails unless the entire range has already been reserved. The resulting error code is `ERROR_INVALID_ADDRESS`.

We are using both `MEM_COMMIT` and `MEM_RESERVE`, and our `lpAddress` is not null.

Let's try reserving a memory region, but this time, from the end of the region (instead of from the beginning):

```rust
let addr = (region.BaseAddress as usize + region.RegionSize) - 2048;
match process.alloc(addr, 2048) {
    Ok(addr) => {
        println!("Bingo: {:x}", addr);
        process.dealloc(addr);
    }
    Err(_) => {
        println!("Nope");
    }
}
```

```
Bingo: ffff0000
```

Hey, that's… the same value Cheat Engine writes to! At the very last[^1], we can allocate memory where we can inject our assembled code.

## Code injection

Now, we could go as far as getting our hands on some assembler, such as [NASM], and invoke it on the input the user wishes to replace. Then we could read the output bytes of the assembled file, and write it to the desired memory location. However… that's just a lot of tedious work that won't teach us much (the Rust documentation already does an excellent job at teaching us how to work with files and invoke an external process). So I am going to cheat and hardcode the right bytes to complete this step of the tutorial.

Here's what Cheat Engine says the area we're going to patch with the jump looks like:

```
Tutorial-x86_64.exe+2D4F0 - 90                    - nop
Tutorial-x86_64.exe+2D4F1 - 8B 9E E0070000        - mov ebx,[rsi+000007E0]
Tutorial-x86_64.exe+2D4F7 - 83 AE E0070000 01     - sub dword ptr [rsi+000007E0],01
Tutorial-x86_64.exe+2D4FE - 48 8D 4D F8           - lea rcx,[rbp-08]
Tutorial-x86_64.exe+2D502 - E8 19B9FDFF           - call Tutorial-x86_64.exe+8E20
```

Here's the after:

```
Tutorial-x86_64.exe+2D4F0 - 90                    - nop
Tutorial-x86_64.exe+2D4F1 - 8B 9E E0070000        - mov ebx,[rsi+000007E0]
Tutorial-x86_64.exe+2D4F7 - E9 042BFCFF           - jmp FFFF0000
Tutorial-x86_64.exe+2D4FC - 66 90                 - nop 2
Tutorial-x86_64.exe+2D4FE - 48 8D 4D F8           - lea rcx,[rbp-08]
Tutorial-x86_64.exe+2D502 - E8 19B9FDFF           - call Tutorial-x86_64.exe+8E20
```

```
FFFF0000 - 83 86 E0070000 02     - add dword ptr [rsi+000007E0],02
FFFF0007 - E9 F2D40300           - jmp Tutorial-x86_64.exe+2D4FE
```

Let's finish up this tutorial step. Don't worry though, the addresses will still be correctly calculated. It's just the opcodes for the ADD instruction and NOP, mostly:

```rust
let region = process
    .memory_regions()
    .into_iter()
    .rev()
    .find(|p| (p.State & winnt::MEM_FREE) != 0 && (p.BaseAddress as usize) < addr)
    .unwrap();

let target_addr = process.alloc(region.BaseAddress as usize + region.RegionSize - 2048, 2048).unwrap();

// The relative JMP itself are 5 bytes, the last 2 are NOP (hence the -2 in delta calculation).
// Relative jumps add to the instruction pointer when it *ends* executing the instruction (like JMP).
//   jmp target_addr
//   nop 2
let mut jmp = [0xE9, 0, 0, 0, 0, 0x66, 0x90];
jmp[1..5].copy_from_slice(&((target_addr as isize - (addr - 2) as isize) as i32).to_le_bytes());
process.write_memory(addr - jmp.len(), &jmp).unwrap();

// addr is already where the old instruction ended, no need to re-skip our previously written jump.
// By the end of the execution of this jump, the instruction pointer will be at (base + code len).
//   add dword ptr [rsi+000007E0], 2
//   jmp addr
let mut injection = [0x83, 0x86, 0xE0, 0x07, 0x00, 0x00, 0x02, 0xE9, 0, 0, 0, 0];
let inj_len = injection.len();
injection[8..12].copy_from_slice(&((addr as isize - (target_addr + inj_len) as isize) as i32).to_le_bytes());
process.write_memory(target_addr, &injection).unwrap();

println!("Replaced the SUB 1 at {:x} with ADD 2 at {:x} successfully!", addr, target_addr);
```

So there we have it! The code calculates the correct relative address to jump to, depending on wherever the breakpoint was hit and wherever we ended up allocating memory. It also places in the ADD instruction, and this is enough to complete this tutorial step!

## Other code injection techniques

We have seen one way to inject more than enough code for most needs (just allocate as much as you need!), through the use of watchpoints to figure out where the offending code we want to patch is. But this is not the only way!

There are things known as "Windows hooks" which allow us to inject entire DLLs (Dynamic Loaded Libraries). We could also try mapping an existing program into the address space of the victim thread. Or we could create a remote thread which loads the library. Here's the more detailed [Three Ways to Inject Your Code into Another Process][three-inject] article.

When writing this post, I discovered other things, such as [what the `SE_DEBUG_NAME`][se-dbg] was and if I needed it, [why `VirtualAlloc` was failing][valloc-fail] or [why could it be failing][valloc-fail-why], [what the error code meant][valloc-errcode], among a couple other things. So there is definitely a lot to learn about this topic[^2].

## Finale

This post was a bit bittersweet for me! One takeaway definitely is the need to be a bit more creative when it comes down to studying how a different program works, but after all, if Cheat Engine can do it, so can we. There are still some unknowns left, and some shortcuts which we could've avoided, but regardless, we've seen how we can make it work. Making it ergonomic or more customizable comes later. Really, sometimes you just need to [embrace the grind][grind] and get a first working version out. Don't obsess with making it perfect or cleaner at first, it's such a waste of time (if you *are* going to clean it up in the end, plan ahead, estimate how long it would take, and put aside your changes until the cleaning is done).

The [code for this post][code] is available over at my GitHub. You can run `git checkout step7` after cloning the repository to get the right version of the code. Again, only the code necessary to complete the step is included at the `step6` tag.

In the next post, we'll tackle the eighth step of the tutorial: Multilevel pointers. This step is what actually got me inspired into starting this entire series, which is why you may have felt this entry a bit more rushed. It is fairly more complicated than [part 6](/blog/woce-6) with a single pointer, because there's some ingenious work that needs to be done in order to efficiently, and automatically, solve it. I didn't manage to figure it out before starting the series, but maybe we're prepared now?

The next post will also be the second-to-last entry in this series (the last step looks pretty tough as well!). After that, there are bonus levels of an actual graphical game, but as far as I can tell, it's there to gain a bit more experience with something more "serious", which I will probably leave as an exercise to the reader.

### Footnotes

[^1]: That "little" hiccup of me trying to figure out how Cheat Engine was finding that precise working location is what put an end to my one-blog-per-week streak. Ah well, sometimes taking a break from something and coming back to it later on just makes the problem obvious (or in this case, a new simple idea which happened to work).

[^2]: I'm still not sure why we could not allocate near the first bytes of the free region, but we could do so just fine near the end.

[unintended]: https://github.com/preames/public-notes/blob/master/unintended-instructions.rst
[global-alloc]: https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-globalalloc
[local-alloc]: https://docs.microsoft.com/en-us/windows/desktop/api/WinBase/nf-winbase-localalloc
[heap-alloc]: https://docs.microsoft.com/en-us/windows/desktop/api/HeapApi/nf-heapapi-heapalloc
[virtual-alloc]: https://docs.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualalloc
[`VirtualAllocEx`]: https://docs.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualallocex
[allocs]: https://docs.microsoft.com/en-us/windows/win32/memory/comparing-memory-allocation-methods
[memprot]: https://docs.microsoft.com/en-us/windows/win32/memory/memory-protection-constants
[NASM]: https://nasm.us/index.php
[three-inject]: https://www.codeproject.com/Articles/4610/Three-Ways-to-Inject-Your-Code-into-Another-Proces
[se-dbg]: https://duckduckgo.com/?t=ffcm&q=SE_DEBUG_NAME&ia=web
[valloc-fail]: https://forum.exetools.com/showthread.php?t=8963
[valloc-fail-why]: https://stackoverflow.com/a/21683133/
[valloc-errcode]: https://social.msdn.microsoft.com/Forums/en-US/4ccf4dd8-eb43-4f5e-8860-c588d6a4f880/virtualallocex-memreserve-pagereadwrite-has-failed-with-system-error-code-487
[grind]: https://jacobian.org/2021/apr/7/embrace-the-grind/
[code]: https://github.com/lonami/memo
