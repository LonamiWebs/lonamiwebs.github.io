+++
title = "Writing our own Cheat Engine: Multilevel pointers"
date = 2021-08-20
updated = 2021-10-17
[taxonomies]
category = ["sw"]
tags = ["windows", "rust", "hacking"]
+++

> Or: Dissecting Cheat Engine's Pointermaps

This is part 8 on the *Writing our own Cheat Engine* series:

* [Part 1: Introduction](/blog/woce-1) (start here if you're new to the series!)
* [Part 2: Exact Value scanning](/blog/woce-2)
* [Part 3: Unknown initial value](/blog/woce-3)
* [Part 4: Floating points](/blog/woce-4)
* [Part 5: Code finder](/blog/woce-5)
* [Part 6: Pointers](/blog/woce-6)
* [Part 7: Code Injection](/blog/woce-7)
* Part 8: Multilevel pointers

In part 7 we learnt how to allocate memory in the remote process, and how we can use that memory to inject our own code for the remote process to execute. Although we didn't bother with an assembler, it shows just how strong this technique can really be. With it we've completed the Read, Write and eXecute trio!

Now it's time to find how we can make our work persist. Having to manually find where some value lives is boring. If the game is able to find the player's health for its calculations, no matter how many times we restart it, then why can't we?

This entry will review how Cheat Engine's pointermaps work, analyze them, and in the end we will approach the problem in our own way. Because this post is quite lengthy, here's a table of contents:

* [Multilevel pointers](#multilevel-pointers)
* [Pointers pointing points](#pointers-pointing-points)
* [Pointer maps](#pointer-maps)
* [Single-threaded naive approach](#single-threaded-naive-approach)
* [Speeding up the scan](#speeding-up-the-scan)
* [Working out a PoC](#working-out-a-poc)
* [Doing more for better runtime speed](#doing-more-for-better-runtime-speed)
* [Doing less for better runtime speed](#doing-less-for-better-runtime-speed)
* [Retrospective](#retrospective)
* [Finale](#finale)

## Multilevel pointers

<details open><summary>Cheat Engine Tutorial: Step 7</summary>

> This step will explain how to use multi-level pointers.
>
> In step 6 you had a simple level-1 pointer, with the first address found already being the real base address. This step however is a level-4 pointer. It has a pointer to a pointer to a pointer to a pointer to a pointer to the health.
>
> You basicly do the same as in step 6. Find out what accesses the value, look at the instruction and what probably is the base pointer value, and what is the offset, and already fill that in or write it down. But in this case the address you'll find will also be a pointer. You just have to find out the pointer to that pointer exactly the same way as you did with the value. Find out what accesses that address you found, look at the assembler instruction, note the probable instruction and offset, and use that, and continue till you can't get any further (usually when the base address is a static address, shown up as green).
>
> Click Change Value to let the tutorial access the health. If you think you've found the pointer path click Change Register. The pointers and value will then change and you'll have 3 seconds to freeze the address to 5000.
>
> Extra: This problem can also be solved using a auto assembler script, or using the pointer scanner.
>
> Extra2: In some situations it is recommended to change ce's codefinder settings to Access violations when encountering instructions like mov eax,[eax] since debugregisters show it AFTER it was changed, making it hard to find out the the value of the pointer.
>
> Extra3: If you're still reading. You might notice that when looking at the assembler instructions that the pointer is being read and filled out in the same codeblock (same routine, if you know assembler, look up till the start of the routine). This doesn't always happen, but can be really useful in finding a pointer when debugging is troublesome.

</details>

## Pointers pointing points

If you say "pointer" enough, you'll end up having [semantic satiation][semsat]. My goal by the end of this post is that you actually get to experience that phenomenon. Anyway, no real program would actually have pointers pointing to pointers which themselves point to a different point (and, you've guessed it, this point points to another pointer pointing to yet another pointer), right? That would be silly. Why would I have a value behind, say, 5 references? I'm not writing Rust code like `&&&&&value`.

But I am sure you are much more likely to be doing something like `game.world.areas[i].players[j].regen()`. And there's a lot of references there:

```
 game.world.areas[i].players[j].regen()
^    ^           ^^^        ^^^      ^^
|    |            |          |        \called by &mut ref
|    |            |           \taking by &mut ref
|    |             \taking by &mut ref
|     \accessing by &mut ref
 \this game is actually in a `Box` (so you're accessing it behind other ref)
```

Each of those is a different structure, with many fields each (for example, the areas also contain enemies and items dropped in different vectors). When there's more than one field, the pointer often points <span class="dim">(sorry)</span> to the beginning of the structure, and you need to add some offset to reach the desired field.

```rust
#[repr(C)] // <- used for clarity to get precise offsets
struct Area {
    /* offset 00 */ pub monsters: Vec<Monster>,
    /* offset 24 */ pub items: Vec<Item>,
    /* offset 48 */ pub kill_goal: u32,
    /* offset 52 */ pub players: Vec<Player>,
}
```

If you have a reference to some `&Area` but access the `players` field, you actually need to read from `[addrof area + 52]`. This is why the tutorial step suggests to "look at the instruction", because it very likely encodes the offset somewhere (if not directly, nearby). Looking at instructions to determine offsets works because normally people want their games to be fast, so they make good use of the available CPU instructions. Obfuscating hot code could slow a game way too much (but it may still be done to some degree!).

To complicate things further, the same reference to one thing may be stored in multiple locations, making it possible to find your goal address through many different paths. In Rust, this happens when you have a shared pointer, such as `Rc<T>` or `Arc<T>` (or if you go the `unsafe` route and have the same `*const T` value scattered around).

The tutorial suggests to complete this step in the same way we did back in step 6. Add a watchpoint, find out what code is accessing this address, look around the disassembly, and write down your findings. Although this technique definitely is a valid way to approach the problem, it is quite tedious and error-prone. It would be hard to fully automate this, because who knows what shenanigans the code could be doing to calculate the right pointer and offset! Sure, Cheat Engine's tutorial is not going to purposedly obfuscate the instructions manipulating our target address. But other programs may be dynamically reading the offset from somewhere.

This technique is also pretty intrusive, because it requires us to attach ourselves as the debugger of the victim program. I hardly have any experience writing debuggers, leave alone writing them in a way that makes them hard to detect! I'm sure it's a very interesting topic, but it's not the current topic at hand, so we'll leave it be. If you know of good resources for this, let me know so I can link them here. Furthermore, we've already gone this route before, so it would be silly to repeat that here, just to end up with a longer version of it.

You may have noticed the "extra" information the tutorial step provides:

> Extra: This problem can also be solved using a auto assembler script, or using the pointer scanner.

We've already done the "auto assembler script" part before (in part 7). I'm not sure how you would approach this problem with that technique. Maybe one could dig until the base pointer, and replace whatever read is happening there with a hardcoded value so that the game thinks that's what it actually read? I'm not sure if it would be possible to solve with injected code without following the entire pointer chain. Or maybe you could instead patch the write to use a fixed value. But anyway, we're not doing that, no manual work will happen on this one. No, we're interested in the <span class="rainbow">pointer scanner</span>[^1].

## Pointer maps

Once you find a value in Cheat Engine, you have the option to "Generate pointermap". This will prompt you to select a file where the generated pointermap will be stored, in `.scandata` format (along with its `.addresslist`). If you're scanning a lot of memory, you will get to see a progress window (otherwise, it will be pretty much instant), along with some statistics:

* Unique pointervalues in target.
* Scan duration.
* Paths evaluated.
* Paths / seconds.
* Static and dynamic queue sizes.
* Results found.
* Time spent writing.
* Lowest known path.

My guess for "unique pointervalues" is the set of pointers found so far, and the queues may be used by the way the scan is done, presumably hinting at an implementation detail. The rest of information is pretty much self-explanatory (lowest known path probably is the shortest "pointer path" found so far). When I talk about "pointer paths", I'm referring to a known, static base address that won't change, with a list of offsets that, when followed, arrive at some desired value in memory (for example, your character's health). In essence, it's a path made out of pointers, with a new pointer to follow at each step. The solution found with Cheat Engine for this tutorial step makes for a good example:

```rust
let offsets = [10, 18, 0, 18]; // list of offsets
let mut addr = EXE_BASE_ADDR + 0x00306B00; // current addr (initialized to base addr)

// follow the path:
// addr_at("Tutorial-x86_64.exe"+00306B00) -> 0165F260
// addr_at(10+0165F260) -> 01690000
// addr_at(18+01690000) -> 01677790
// addr_at( 0+01677790) -> 01601A80
//         18+01601A80  -> 01601A98
for offset in offsets {
    addr = process.read_addr_at(addr);
    addr += offset;
}

let value = process.read_desired_value_at(addr);
```

Let's get back to talking about Cheat Engine's scan. After generating the pointermap, the idea is to force the game to change the pointer path (for example, by closing and re-opening the game again) and find your target value once again. For the tutorial, we can just change the pointer. After we find the value again, we do a "Pointer scan for this address". The "Pointerscanner options" has a checkbox to "Compare results with other saved pointermap(s)". Running this seems to generate a second pointermap, and after some magic, both are compared and the one true pointer path is found[^2].

There's a bunch of files generated:

* `.scandata` is a bunch of binary data that I have no idea what could contain.
* `.scandata.addresslist` seems to contain `ADDRESS=DESCRIPTION`,
 one per line, of the addresses you had "saved" when the first
pointermap was made. This seems to be used when performing the pointer
scan and comparing results (so that you can choose the address you want
to compare it to).
* `.PTR` is 1201 bytes (such an strange size) and seems to contain a list of the modules loaded by the program
* `.PTR.results.#`, where `#` is a number between 0 and 8, are mostly empty files (except for 4 which is 14 bytes).

Now, there's this one option under "advanced" known as "Compress pointerscan file". The long description reads (emphasis mine):

> Compresses the generated .PTR files *slightly*, so they take less space on the disk and less time writing to disk. Most of the time the bottleneck of a pointerscan is disk writing, so it is recommended to use this option.

Slightly, huh. Well, for the tutorial, which is using (according to the task manager) 2'364 K, running the scan with the compression disabled generates roughly *5 gigabytes* across the nine `.PTR.results`. That's… not too shabby for a "slight" compression.

Let's guess what those files are storing. The screen with the results does say it found uh, well you know, the usual, 122'808'639 pointer paths. This is the result of scanning for an address. That's (very) roughly 40 bytes per path, and assuming 8 bytes for each address/offset, equates to 5 hops. I guess the math kind of checks out?

On the other hand, "generate pointermap" just spits out the `.scandata` at roughly 60KB. So these two options are definitely doing something very, very different. And I have no idea what either of these are doing. Let's dive into Cheat Engine's "advanced options" for the pointer scan to try and gain some insight. I will be listing all the settings available in the scan form and adding a bit on whether I think they're useful to us or not.

# Scan options

The *Pointerscanner scanoptions* window has plenty of options that are extremely valuable to gain insight of what's going on behind the scenes without having to dig into the code. At the very top we have three modes:

* Scan for address
* Scan for addresses with value
* Generate pointermap

The third option is what we use during the first step, and the first option for the second step.

When using either the first or second mode, you can also check *Use saved pointermap* which you can use if you have created a pointermap on a system that runs the game, but you wish to do the scan on another system (or multiple systems).

With the first or second mode, you can also *Compare results with other saved pointermap(s)* which, when ticked, lets you add other pointermaps which will be used to verify that the pointers it finds are correct. You do have to fill in the correct address for each pointermap provided, and one should expect at least the size of the game itself in memory for every pointermap used. We know this step is key, but we don't know how that comparison could be possibly done.

The checkbox *Include system modules* I presume also scans in system modules and not just game's own modules, which is useful if you suspect the value lives elsewhere. Not helpful for us right now, but good to know this is a possibility.

Apparently, Cheat Engine can improve pointerscan with gathered heap data. The heap is used to figure out the offset sizes, instead of blindly guessing them. This should greatly improve speed and a lot less useless results and give perfect pointers, but if the game allocates gigantic chunks of heap memory, and then divides it up itself, this will give wrong results. If you only allow static and heap addresses in the path, when the address searched isn't a heap address, the scan will return 0 results. I do not really know how Cheat Engine gathers heap data here to improve the pointerscan, but since this mode is unchecked by default, we should be fine without it.

By default, the pointer path may only be inside the region 0000000000000000-7FFFFFFFFFFFFFFF. There's a fancier option to limit scan to specified region file, which presumably enables a more complex, discontinuous region. Or you can filter pointers so that they end with specific offsets[^15]. Or you can indicate that the base address must be in specific range, which will only mark the given range as valid base address (this reduces the number of results, and internally makes use of the "Only find paths with a static address" feature by marking the provided range as static only, so it must be enabled).

Pointers with read-only nodes are excluded by default, so the pointerscan will throw away memory that is readonly. When it looks for paths, it won't encounter paths that pass through read only memory blocks. This is often faster and yields less useless results, but if the game decides to mark a pointer as readonly Cheat Engine won't find it.

Only paths with a static address are "found". The pointerscan will only store a path when it starts with a static address (or easily looked up address). It may miss pointers that are accessed through special paths like thread local storage (but even then they'd be useless for Cheat Engine as they will change). When it's disabled, it finds every single pointer path. Now, this bit is interesting, because the checkbox talks about "find", but the description talks about "store", so we can guess there's no trick to only "finding" correct ones. It's going to find a lot of things, and many of them will be discarded. It also mentions thread-local storage and how we probably shouldn't worry about it.

Cheat Engine won't stop traversing a path when a static has been found by default. When the pointerscanner goes through the list of pointervalues with a specific value, this will stop exploring other paths as soon as it encounters a static pointer to that value. By enabling this option, some valid results could be missed. This talks about "pointervalues with a specific value", which is a bit too obscure for me to try and make any sense out of it.

Addresses must be 32-bit alligned. Only pointers that are stored in an address dividable by 4 are looked at. When disabled, it won't bother. It enables fast scans, but "on some horrible designed games that you shouldn't even play it won't find the paths". Values in memory are often aligned, so reducing the search space by 75%[^16] is a no-brainer.

Cheat Engine can optionally verify that the first element of pointerstruct must point to module (e.g vtable). Object oriented programming languages tend to implement classobjects by having a pointer in the first element to something that describes the class. With this option enabled, Cheat Engine will check if it's a classobject by checking that rule. If not, it won't see it as a pointer. It should yield a tremendous speed increase and almost perfect pointers, but it doesn't work with runtime generated classes (Java, .NET). Optionally, it can also accept non-module addresses. I have no idea how this is achieved, but since it's disabled by default, we should be able to safely ignore it.

By default, no looping pointers are allowed. This will filter out pointerpaths that ended up in a loop (for example, base->p1->p2->p3->p1->p4 since you could just as well do base->p1->p4 then, so throw this one away (base->p1->p4 will be found another way)). This gives less results so less diskspace used, but slightly slows down the scan as it needs to check for loops every single iteration. The thought of how much data the 5GB scan would generate without this option makes me shiver.

Cheat Engine will allow stack addresses of the first thread(s) to be handled as static, which allows the stack of threads to be seen as static addresses by the pointerscan. The main thread is always a sure bet that it's the first one in the list. And often the second thread created is pretty stable as well. With more there's a bigger chance they get created and destroyed randomly. When a program enters a function and exits it, the stack pointer decreases and increases, and the data there gets written to. The farther the game is inside function calls, the more static the older data will be. With max stack offset you can set the max size that can be deemed as static enough (the max stackoffset to be deemed static enough is 4096 by default). It finds paths otherwise never found, but since there are more results, there's more diskspace.

Cheat Engine by default will look at the stacks of two threads, from oldest to newest. It indicates "the total number of threads that should be allowed to be used as a stack lookup. Thread 1 is usually the main thread of the game, but if that one spawns another thread for game related events, you might want to have that secondary thread as well. More threads is not recommend as they may get created and destroyed on the fly, and are therefore useless as a lookup base, but it depends on the game".

Unfortunately, this option is enabled by default, so it seems pretty important, and we might need to put some work into figuring out how "stacks" are found. However, this would mean that some "base" object (like a `Game` instance) is passed down by reference hundreds of calls, which seems pretty annoying just to have access to something that effectively acts like a global, so hopefully games don't make use of this.

This can be taken a step further, and consider stack addresses as ONLY static address, if you wish to only find pointer paths with a stack address. It must be combined with "Only find paths with a static address" (default on) else this option will have no effect. You'll only get paths from the stack, but you don't get get paths from random DLL's or the executable.

The pointerscan file is by default compressed. Cheat Engine Compresses the generated .PTR files slightly so they take less space on the disk and less time writing to disk. Most of the time the bottleneck of a pointerscan is disk writing, so it is recommended to use this option (which was not available in older versions).

Only positive offsets are scanned by default, but Cheat Engine may optionally scan for negative offsets as well (although it can not be used in combination with compressed pointerscan files; this seems to hint that the compression assumes only positive values).

On my machine, 9 threads are scanning by default with a maximum offset value of 4095 and a maximum level (depth) of 7. The maximum different offsets per node are 3. When the pointerscan looks through the list of pointers with a specific value, it goes through every single pointer that has that value. Every time increasing the offset slightly. With this feature enabled the pointerscan will only check the first few pointers with that value. This is extremely fast, and the results have the lowest pointer paths possible, but you'll miss a lot of pointers that might be valid too. I think this description is key, as it clearly says what the pointerscan does and maybe even how it works (although it sounds a bit inefficient, so Cheat Engine probably uses other tricks).

Cheat Engine clearly knows this process is expensive, so it optionally allow scanners to connect at runtime. This opens a port that other systems running the pointerscanner can connect to and help out with the scan. Or it can connect to pointerscan node, which will send a broadcast message on the local network which will tell pointer scanner systems to join this scan if they are set to auto join (or "Setup specific IP's to notify" to notify systems of this scan that are outside of the local network).

And that's all! In summary:

* Assume addresses are 32-bit aligned (maybe even 64-bit).
* Discard paths that don't end in a static address (bonus points if the top of the stack for the firsts two threads are also considered).
* Ignore read-only memory.
* Limit the number of offsets per pointer to something small like 3, and give up after reaching a depth greater than 7.
* Limit the offset range to `0..4096`.
* Use multiple threads.

## Single-threaded naive approach

After playing around a bit more with Cheat Engine's scans, I realized the 14 bytes of the `.PTR.results.4` is because the process literally finds a single path which it places there. Running the process with compression and no previous scan to compare it to spits out roughly 750MB (so the compression does go from 5GB to 750MB, that's a lot more reasonable).

In any case, we're with the `.scandata` now. I really do wonder what could it possibly contain? I really doubt it's the pointer paths found, because then it would be huge. Perhaps it contains the memory regions? That would make some sense, since the sibling `.addresslist` *is* a list of all the loaded modules. Maybe the `.scandata` contains the memory regions for all of those loaded modules.

For the first time in this series, I really don't know how Cheat Engine could be working behind the scenes. Is it really evaluating millions of *paths*? That's a lot of memory, no matter how you encode it! I'm really impressed at the processing speed if this is in fact the case. Let's see how a naive approach for that could look like[^3].

We start off with a single address, the address of a particular value we care about in memory (for example, the player's health). This address is an 8-byte number (which for us is an `usize`), so we can look for pointer-values (values in memory that look like a pointer to a certain address) that point to this address (or close enough). Let's call this `goal_addr`.

For every memory block, and for every pointer value `ptr_val` in it, we check if the distance between the `ptr_val` and the `goal_addr` falls within an arbitrary range, for example:

```rust
let process = Process::open(pid)?;

let mask = winnt::PAGE_EXECUTE_READWRITE
    | winnt::PAGE_EXECUTE_WRITECOPY
    | winnt::PAGE_READWRITE
    | winnt::PAGE_WRITECOPY;

let regions = process
    .memory_regions()
    .into_iter()
    .filter(|p| (p.Protect & mask) != 0) // (1)
    .collect::<Vec<_>>();

let mut candidate_locations = Vec::new();

for region in regions {
    let base = region.BaseAddress as usize;
    let block = match process.read_memory(base, region.RegionSize) {
        Ok(block) => block,
        Err(_) => continue, // (2)
    };

    for (offset, chunk) in block.chunks_exact(8).enumerate() { // (3)
        let ptr_val = usize::from_ne_bytes(chunk.try_into().unwrap()); // (4)
        if (0..4096).contains(goal_addr.wrapping_sub(ptr_val)) { // (5)
            let ptr_val_addr = base + offset * 8;
            candidate_locations.push(ptr_val_addr); // (6)
        }
    }
}
```

There's a lot of things to unpack in this small snippet:

1. We're only interested in regions that are both readable and writable, pretty much like Cheat Engine is doing.
2. If we can't read a memory region, we can just skip it. Our desired address is probably not there. There's a lot of regions anyway so this is probably a good thing as we can reduce the scanning time!
3. `chunks_exact` achieves multiple things:
    * It's the most concise way to read chunks of 8 bytes in size, the alternative being having a `for i in (0..block.len())` and then slicing on `&block[i..i+8]`.
    * It will look on aligned addresses[^4] for free (the alternative being `.windows(8)`, which would also look for unaligned addresses).
    * It makes sure the chunk is always 8 bytes in size, which is important[^5], because `usize` is also 8 bytes in size on 64-bit machines.
4. Interpreting 8 bytes of memory as an `usize` can be safely (and efficiently!)[^6] achieved through `usize::from_ne_bytes`, which expects an `[u8; mem::size_of::<usize>()]`. Thankfully, we can convert the 8-byte-long slice into an array pretty easily with `.try_into()`.
5. It's important to use `wrapping_sub`, because the `-` operator would panic on underflow on debug by default[^7]. Since we're reading all of the memory in the program, there will be a lot of values, many of which would be less than `goal_addr`, causing underflow. Note also how we could interpret the values as `isize` instead so that a negative offset could be used in the range. However, a negative offset is much less common[^8], so it's fine to stick with positive offsets.
6. We have a candidate pointer-value, so we make sure to store its address.

At the end of this, `candidate_locations` will have *many* memory addresses pointing to a different `ptr_val` each. This `ptr_val` points to `goal_addr` minus some offset (which can be calculated at any time by substracting again). These are the pointer-values at depth 0[^9].

Each of these `candidate_locations` is in itself the next `goal_addr`, and running the process again will produce pointer-values for depth 1. Yes, you've guessed it, this has exponential growth. No wonder Cheat Engine finds millions of paths. And don't forget to somehow save "this address came from this other address", so that you can follow the chain back after you're done!

Note the importance of limiting the depth: not only this growth has to stop at some point, but also think about cyclic paths. The program would get stuck as soon as `ptr_val_addr == ptr_val`, looking for itself over and over again! Without actively looking for cycles[^10], and without limiting the depth, the process would never finish.

After the full process completes (having executed multiple iterations of it at multiple depths), we would need to check every path to see if it works for us (that is, if it starts with a "static address"). You will have an obscene amount of paths, many of which won't actually work after restarting the program (it might have been luck that some unrelated component got allocated close to your original `goal_addr` but now it's not anymore). So how do we clean this mess up?

We run the process again! Preferably, after the memory has shuffled around enough (for example, again, restarting the program). Once we have the list of paths "before" and "after", we compare them all. The naive approach of checking, for every path in "before", if any of the paths in "after" is the same, would yield a sweet time complexity of `O(n²)`, with millions of paths. This ain't gonna cut it. We must do better. I don't know if this is what Cheat Engine is doing (but if it is, I tip my hat to them), but since I can't think of an efficient way to do it, we'll be going a different route.

## Speeding up the scan

By reading an entire block of memory at a time, we're actually doing pretty okay on that department. It would be very, very wasteful to issue millions of reads of 8 bytes, when we could instead run thousands of reads of several kilobytes (or more!). Of course, we still have to read millions of 8-bytes, but if they're in our memory and don't require a call to the Windows API, it's going to be orders of magnitude faster.

We're only reading aligned pointers, cutting down the amount of reads and checks we perform down to `1/8`. A lot of useless results are also discarded this way.

We're only considering positive offsets, and we're limiting how "far" the `goal_addr` can be from a possible `ptr_val` before we stop considering said `ptr_val`. After all, a structure longer than 4096 bytes should hopefully be uncommon. By doing this, we only keep "address-like" values, which have a very high chance of being an actual address, although they could very well not be! We may be finding arbitrary values and think they represent an address when they actually don't.

We're limiting the maximum depth we're willing to go. This depth directly correlates to the maximum length a pointer path can have. If you're confident the path won't be longer than, say, 5 addressses, there's no need to dive any deeper, and you will save on a lot of processing this way.

This code can be made parallel trivially (after making Rust compiler happy, anyway). There is a lot of `ptr_val_addr` values to scan for, so if we think of `candidate_locations` as a "queue of work", more than one thread can be popping from it and running the scan. This gives a nice boost on multi-core systems. It doesn't entirely scale linearly with the number of cores, but it's close enough to what you would expect.

A pointer path will only be considered if it starts with a static address. This means the last address pushed must be static (the path will have been built backwards, because we started at the end, `goal_addr`). This should clean-up a lot of intermediate and uninteresting addresses. If the address isn't static, it's not really interesting to us. Remember, the reason we're doing all of this is so that we can reuse said address in the future, without the need to find `goal_addr` manually.

Comparing the pointer paths will result in paths that very likely will work in the future. Not only is this important to reduce the number of paths drastically, but it also provides better guarantees about what is a "good", reliable path to follow to find `goal_addr`.

Next up, let's talk about some of the more intrusive optimizations which I actually seeked to reach an acceptable runtime. This will be where I started to code this up.

## Working out a <abbr title="Proof of Concept">PoC</abbr>

> Add a braindump mess enough to find pointerpaths

This is the commit message that made it possible to complete step 8 on the tutorial (the actual commit message has quite some more lines explaining the commit). Unlike previous entries of this series, I had a hard time making incremental progress. So let's dissect what was done instead.

> The approach used in this commit (although really messy), consists on taking two "snapshots" of the memory, and knowing where a desired value is located in both.

By introducing the concept of "snapshots", we can "freeze" the process' memory at a given point in time, and scan it at our leisure, without having to worry about it changing. Not only this, but it also saves on a lot of calls to `ReadProcessMemory`, so it's also more efficient. If memory is an issue, these structures could be saved to disk and streamed instead. I haven't measured how fast this is, but having our own copy of the process' memory lets us run the scan even after the process is closed (and by then we would reclaim some of that memory), so this approach is mostly benefits.

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub real_addr: usize,
    pub mem_offset: usize,
    pub len: usize,
    pub base: bool, // is this a "base" block (i.e. `real_addr` will be static)?
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub memory: Vec<u8>,
    pub blocks: Vec<Block>,
}

impl Snapshot {
    pub fn new(process: &Process, regions: &[winapi::um::winnt::MEMORY_BASIC_INFORMATION]) -> Self {
        // These are used to determine "base" blocks.
        let modules = process.enum_modules().unwrap();

        // Adapt all regions used by the program into our friendlier structure.
        let mut blocks = regions
            .iter()
            .map(|r| Block {
                real_addr: r.BaseAddress as usize,
                mem_offset: 0,
                len: r.RegionSize,
                // "base" blocks are those where they start at the same address
                // as some module, as seen in the sixth entry of this series.
                base: modules.iter().any(|module| {
                    let base = r.AllocationBase as usize;
                    let addr = *module as usize;
                    base == addr
                }),
            })
            .collect::<Vec<_>>();

        // This will come in useful later.
        blocks.sort_by_key(|b| b.real_addr);

        // Put all the memory in a flat vector. The blocks will tell us where each index belongs.
        let mut memory = Vec::new();
        let blocks = blocks
            .into_iter()
            .filter_map(|b| match process.read_memory(b.real_addr, b.len) {
                Ok(mut chunk) => {
                    let len = chunk.len();
                    let mem_offset = memory.len();
                    memory.append(&mut chunk);
                    Some(Block {
                        real_addr: b.real_addr,
                        mem_offset,
                        len,
                        base: b.base,
                    })
                }
                Err(_) => None,
            })
            .collect::<Vec<_>>();

        Self {
            memory,
            blocks,
        }
    }
}
```

Pretty straightforward. A `Snapshot` consists of the process' memory along with some metadata for the blocks. This lets us know, given an index into `memory`, what its real address (or vice versa):

```rust
fn get_block_idx_from_mem_offset(&self, mem_offset: usize) -> usize {
    match self.blocks.binary_search_by_key(&mem_offset, |b| b.mem_offset) {
        Ok(index) => index,
        Err(index) => index - 1,
    }
}

fn get_block_idx_from_addr(&self, addr: usize) -> usize {
    match self.blocks.binary_search_by_key(&addr, |b| b.real_addr) {
        Ok(index) => index,
        Err(index) => index - 1,
    }
}
```

Because we've sorted by `real_addr`, and we filled the `memory` in order, we can `binary_search_by_key` in both cases. `Process::read_memory` translates into `Snapshot::read_memory` as follows:

```rust
pub fn read_memory(&self, addr: usize, n: usize) -> Option<&[u8]> {
    let block = &self.blocks[self.get_block_idx_from_addr(addr)];
    let delta = addr - block.real_addr;
    if delta + n > block.len {
        None
    } else {
        let offset = block.mem_offset + delta;
        Some(&self.memory[offset..offset + n])
    }
}
```

Because this time we already own the memory, we can return a slice and avoid allocations[^11]. Now that we have two snapshots of the process' memory at different points in time (so the pointer-values to `goal_addr` are different), we find `goal_addr` in both snapshots (it should be a different pointer-value, unless it so happens to be in static memory already).

Then, the pointer value of the address is searched in the second snapshot (within a certain range, it does not need to be exact). For every value found, a certain offset will have been used. Now, the pointer value minus *this exact offset* **must** be found *exactly* on the other snapshot (it does not matter which snapshot you start with[^12]). This was my "aha!" moment, and it's a key step, so let's make sure we understand why we're doing this.

Rather than guessing candidate pointer-values which would have a given offset as a standalone step, we merge this with the comparison step, insanely reducing the amount of candidates. Before, any pointer-value close enough to `goal_addr` had to be considered, and in a process with megabytes or gigabytes of memory, this is going to be a lot. However, by keeping only the pointer-values (which have a given offset) that *also* exist on the alternate snapshot with the *exact* value, we're tremendously reducing the number of false positives.

```rust
impl Snapshot {
    // Iterate over (memory address, pointer value at said address)
    pub fn iter_addr(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        let mut blocks = self.blocks.iter().peekable();
        self.memory
            .chunks_exact(8)
            .enumerate()
            .map(move |(i, chunk)| {
                let mut block = *blocks.peek().unwrap();
                if i * 8 >= block.mem_offset + block.len {
                    // Roll over to the next block.
                    block = blocks.next().unwrap();
                }

                (
                    block.real_addr + (i * 8 - block.mem_offset),
                    usize::from_ne_bytes(chunk.try_into().unwrap()),
                )
            })
    }
}

struct PathFinder {
    first_snap: Snapshot,
    second_snap: Snapshot,
    addresses: std::cell::Cell<Vec<(bool, u8, usize)>>, // (last node?, depth, real address)
}

impl PathFinder {
    fn run(&self, first_addr: usize, second_addr: usize, depth: u8) -> bool {
        // F: first, S: second; RA: Real Address; PV: Pointer Value
        let depth = depth - 1;
        let mut any = false;
        for (sra, spv) in self.second_snap.iter_addr().filter(|(_sra, spv)| {
            if let Some(offset) = second_addr.checked_sub(*spv) {
                offset <= MAX_OFFSET
            } else {
                false
            }
        }) {
            if self.second_snap.is_base_addr(sra) {
                unsafe { &mut *self.addresses.as_ptr() }.push((true, depth + 1, sra));
                any = true;
                continue;
            }
            if depth == 0 {
                continue;
            }
            let offset = second_addr - spv;
            for (fra, _fpv) in self
                .first_snap
                .iter_addr()
                .filter(|(_fra, fpv)| fpv.wrapping_add(offset) == first_addr)
            {
                if self.run(fra, sra, depth) {
                    unsafe { &mut *self.addresses.as_ptr() }.push((false, depth + 1, sra));
                    any = true;
                }
            }
        }

        any
    }
}
```

`Snapshot::iter_addr` is like `read_memory`, but better for our needs, because it automatically returns the pointer-values and its corresponding real address efficiently. The `PathFinder` is a helper `struct` to avoid passing `first_snap`, `second_snap` and `addresses` as parameters on every call.

`Snapshot::run` is a recursive method which is called with the `goal_addr` in both the first and second snapshot, along with a depth. When this depth reaches 0, the method stops recursing. The method also stops when a base (static) address is found.

The method starts by looking for all pointer-values in the second snapshot where `ptr_value + offset = second_addr` for all `offset in 0..=MAX_OFFSET`. For every candidate `ptr_value` with a given `offset`, it looks **exactly** for `first_addr - offset` in the alternate snapshot (the first one). Once found, we have a candidate offset valid in *both* snapshots, and then we can recurse to find subsequent offsets on the real addresses of these pointer values themselves. The addresses of these pointer-values are our new `goal_addr` in the next depth.

Once `run` returns from the top-most depth, we can convert post-process `addresses` into something usable, with an algorithm akin to run-length encoding (the real-code abuses the vector's `capacity` and `len` to determine the `depth` and had inaccurate names, so I've rewritten that part for clarity):

```rust
struct Path {
    addresses: Vec<usize>,
    depth: u8,
}

let mut paths = Vec::new();

for (base, depth, addr) in pf.addresses.into_inner() {
    if base {
        paths.push(Path { addresses: Vec::new(), depth });
    }
    for path in paths.iter_mut() {
        if path.depth == depth {
            path.addresses.push(addr);
            // remember PathFinder started at the highest depth and ended at
            // base with the lowest depth, so "going up" is "the way out".
            path.depth += 1;
        }
    }
}

// `second_addr` wasn't pushed by `PathFinder::run` as it was the starting
// point, so push it now.
for path in paths.iter_mut() {
    path.addresses.push(second_addr);
}
```

Note how this process can form a tree. Any given depth can have any amount of children. For example, if the address finding yields the following addresses (where the hundreds' also represent the depth):

```
400, 300, 450, 300, 200, 100
```

Then this represents the following call-stack tree:

```
   100
    |
   200
    |
   300
   / \
400   450

// or

(100
    (200
        (300
            (400, 450))))
```

Once the many paths have been cleaned up into a separate vector each, we can turn these addresses into offsets:

```rust
paths.into_iter().map(|path| {
    let mut offsets = path.addresses;
    for i in (1..offs.len()).rev() {
        let prev_addr = offs[i - 1];
        let ptr_value = pf.second_snap.read_memory(prev_addr, mem::size_of::<usize>()).unwrap();
        let ptr_value = usize::from_ne_bytes(ptr_value.try_into().unwrap());
        offs[i] -= ptr_value;
    }
    offsets
}).collect::<Vec<usize>>()
```

For the example above, the result would be:

```
100, 100, 100, 100
100, 100, 100, 150
```

In order to reach `address[i]`, we have to read the `ptr_value` from `address[i - 1]` and add a given `offset`. This `offset` is given by `address[i] - ptr_value`. By iterating the list of addresses in reverse, we can neatly turn them into offsets substracting this `ptr_value`. Now we're done! We can persist this list of `offsets` and it will work at any point in the future to get back to our original `goal_addr`. In pseudo-code:

```rust
base = base addr
for offset in offsets[..-1] {
    base = *(base + offset)
}
goal_addr = base + offsets[-1]
value = *goal_addr
```

By the way, sometimes the scan will take horribly long and find thousands of path, and sometimes it will be blazingly fast. I don't know why this is the case, but if that happens, you can try restarting the tutorial. And do not forget to run on `--release` mode, or you will definitely be waiting a long, long time.

## Doing more for better runtime speed

The recursive `PathFinder` implements a fairly elegant solution. Unfortunately, this is hard to parallelize, as it all runs on the same thread and there is no clean way to introduce threads here[^13]. We will rewrite this version to use a queue instead, with the idea that multiple threads will be taking work from it. In order to do this, let's introduce two new concepts:

```rust
#[derive(Clone)]
struct CandidateNode {
    parent: Option<usize>,
    addr: usize,
}

#[derive(Clone)]
struct FutureNode {
    node_idx: usize,
    first_addr: usize,
    second_addr: usize,
    depth: u8,
}
```

The `CandidateNode` should be as small as possible, because there will be one `CandidateNode` for every address of the candidate pointer-values. Without doing anything fancy, we'll need an optional `usize` to build a "linked list" of the path, and the address of the pointer-value. With the `parent` field, we can trace all of the parent candidate nodes all the way back up to the root node.

The `FutureNode` will hold temporary values, until a thread picks it up and carries on, so there's no need to over-optimize this. For a thread to continue, it needs to know the pointer-value address and its parent (that is, the candidate node it will work on), along with the first and second goal address for a given depth.

After the process completes (a base or static address is found), it's enough to remember the candidate node, as we'll later be able to follow the chain. Thus, the `PathFinder` needs to hold the following values:

```rust
struct QueuePathFinder {
    first_snap: Snapshot,
    second_snap: Snapshot,
    /// Indices of `nodes_walked` which are "good" (i.e. have reached a base address).
    good_finds: Vec<usize>,
    /// Shared "tree" of nodes we've walked over, so all threads can access and reference them.
    nodes_walked: Vec<CandidateNode>,
    /// Nodes to be used in the future, where the `FutureNode::node_idx` references `Self::nodes_walked`.
    new_work: Vec<FutureNode>,
}

impl QueuePathFinder {
    pub fn run(&mut self, first_addr: usize, second_addr: usize, depth: u8) {
        self.add_work(None, first_addr, second_addr, depth);
        while self.step() {}
    }

    // Returns false to signal there's no more work.
    fn step(&mut self) -> bool {
        // Instead of getting the `goal_addr` from input parameters, we get it through the queue.
        let future_node = if let Some(future_node) = self.new_work.pop() {
            future_node
        } else {
            return false;
        };

        // The same scan as `PathFinder::run` is carried away, with 2 differences.
        let first_snap = std::mem::take(&mut self.first_snap);
        let second_snap = std::mem::take(&mut self.second_snap);
        for (sra, spv) in second_snap.iter_addr().filter(|(_sra, spv)| {
            if let Some(offset) = future_node.second_addr.checked_sub(*spv) {
                offset <= MAX_OFFSET
            } else {
                false
            }
        }) {
            if second_snap.is_base_addr(sra) {
                // (1) rather than simply pushing the address (here, a `CandidateNode`),
                // we also store its index (because the candidate nodes themselves don't
                // have any flag saying "I'm the bottommost one").
                self.good_finds.push(self.nodes_walked.len());
                self.nodes_walked.push(CandidateNode {
                    parent: Some(future_node.node_idx),
                    addr: sra,
                });
                continue;
            }
            if future_node.depth == 0 {
                continue;
            }
            let offset = future_node.second_addr - spv;
            for (fra, _fpv) in first_snap
                .iter_addr()
                .filter(|(_fra, fpv)| fpv.wrapping_add(offset) == future_node.first_addr)
            {
                // (2) rather than recursing, we add work to the queue.
                self.add_work(Some(future_node.node_idx), fra, sra, future_node.depth - 1);
            }
        }

        self.first_snap = first_snap;
        self.second_snap = second_snap;
        true
    }

    fn add_work(
        &mut self,
        parent: Option<usize>,
        first_addr: usize,
        second_addr: usize,
        depth: u8,
    ) {
        // Adding work consists on registering the `CandidateNode` and adding a `FutureNode`.
        self.new_work.push(FutureNode {
            node_idx: self.nodes_walked.len(),
            first_addr,
            second_addr,
            depth,
        });
        self.nodes_walked.push(CandidateNode {
            parent,
            addr: second_addr,
        });
    }
}
```

This version probably uses more memory, as we need to remember all `CandidateNode` because any live `FutureNode` may be referencing them, and a `CandidateNode` itself has parents. It should be possible to prune them if it gets too large, although a lot of indices would need to be adjusted, so for now, we don't worry about pruning that tree (which we store as a `Vec` and the references to the parent are indirect through the use of indices). However, this version can use threads much more easily. It's enough to wrap all the `Vec` inside a `Mutex`.

And what's more, it is now trivial to perform the search breadth-first instead! With the recursive version, we were stuck performing a depth-first search, which is unfortunate, because the first valid paths which would be found would be the deepest. But now that we have our own work queue, if we keep it sorted by depth, we can easily switch to running breadth-first. Shorter paths feel better, because there's less hops to go through, and less things that could go wrong:

```rust
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)] // <- now it's comparable
struct FutureNode {
    depth: u8, // <- this used to be last but we want to sort by depth first
    node_idx: usize,
    first_addr: usize,
    second_addr: usize,
}

struct QueuePathFinder {
    ...
    new_work: BinaryHeap<FutureNode>,
    //        ^^^^^^^^^^ this used to be a Vec
}
```

Thanks to Rust's wrong decision of making `BinaryHeap` be max-heaps[^14], and our use of a decreasing depth as we get deeper, the ordering just works out! Next up, threads should be introduced for the next big-boost in runtime performance. This isn't too tricky, but I would recommend you introduce `serde` by now and persist both `Snapshot` and `goal_addr` so that you can easily debug this. Running the program on Cheat Engine's tutorial gets boring fast. I'll leave both of these as an exercise to the reader. Just make sure the threads don't end prematurely, because even if there is no work *now*, it doesn't mean there won't be a few milliseconds later. Else you will be back at single-threaded execution!

After adding threads, I kept poking around the program and seeing how seemingly-innocent changes made runtime performance a fair bit worse. Here's some of the insights I got:

* Using `filter` or not (by placing the inverted condition inside the loop with a `continue`) can both help or hurt performance.
* Hoisting certain conditions, like `if depth == 0`, and duplicating the entire loop body rather than running it every time, can hurt performance.
* The moments when you should wake up threads matters (if your approach works in a way where this matters).
* Changing the order in which you compute certain values and then use them can matter.
* `Option` introduces a fair bit of overhead due to alignment concerns, and `CandidateNode` can easily be reduced from 24 bytes to 16 by getting rid of the `Option` and instead using a special value to signal "no-parent".
* Atomics are neat, but a bit annoying to use. Crates like [`crossbeam-utils`] make them easier to use while still not using locks if possible.
* You can beat Rust's functional-style iterators performance by writing your own custom iterator, but it isn't trivial to do so.
* Messing with larger (such as changing `depth` for `usize`) or smaller (such as changing `node_idx` for `u32`) types can hurt performance.

It turns out `step` isn't called a lot while analyzing Cheat Engine's tutorial, so it better be fast. And one way to go fast is to do less!

## Doing less for better runtime speed

For every future node, we have to read and compare an entire snapshot of the process' memory against a value. For 8MiB worth of memory, that's over a million comparisons! Using threads can only scale as far as the amounnt of cores you have before degrading quickly. A lot of those comparisons won't be useful at all, and if the method runs a hundred times, there can easily be 6MiB that you could avoid scanning at all, a hundred times.

What if, instead, we run some sort of "pre-scan" that tells us "don't bother looking around here, you will not find anything useful"? We totally can, and the good news is, it does improve the runtime quite a bit!

In order to do this, we need another way of instructing the program where to look. We can do this by adding additional information to each block (either directly or indirectly) that tells us "which other blocks have pointer-values that point into us?":

```rust
struct Block {
    ...,
    // Indices of the blocks that have pointer-values which point inside self.
    pointed_from: Vec<usize>,
}

pub fn prepare_optimized_scan(snap: &mut Snapshot) {
    let mut block_idx_pointed_from = (0..snap.blocks.len())
        .map(|_| std::collections::HashSet::new())
        .collect::<Vec<_>>();

    // For each block...
    for (i, block) in snap.blocks.iter().enumerate() {
        // ...scan all the pointer-values...
        for (ra, pv) in snap.iter_addr() {
            // ...and if any of the pointer-values points inside this block...
            if let Some(delta) = pv.checked_sub(block.real_addr) {
                if delta < block.len {
                    // ...then we know that the block with this pointer-value points to our original block.
                    block_idx_pointed_from[i].insert(snap.get_block_idx_from_addr(ra));
                }
            }
        }
    }

    // Convert sets into sorted vectors and save them inside the blocks.
    block_idx_pointed_from
        .into_iter()
        .zip(snap.blocks.iter_mut())
        .for_each(|(set, block)| {
            block.pointed_from = set.into_iter().collect::<Vec<_>>();
            block.pointed_from.sort();
        });
}
```

When running the scan (via `Snapshot::step`), instead of running `iter_addr` over *all* addresses, we determine the block where the current `goal_addr` falls in and scan only on the blocks indicated by `block.pointed_from`. I did some math, and on the tutorial step, rather than scanning 95 blocks, we scan an average of 3.145 blocks (median 2, standard deviation 6.12), which greatly reduces the amount of work that needs to be done on a snapshot which is roughly 10 MiB.

There's a chance that the block we're scanning just so happens to be very "busy" and have a lot of blocks pointing into it (which would make sense, as that's probably an indication that the interesting things occur there). However, it is definitely possible to improve on the heuristics, all with different trade-offs.

The simplest heuristic is "assume every block can point to any other block" (which we were doing before). A slightly better one is "determine which blocks have a chance of pointing into other blocks". You could even narrow down the "scan area" within blocks to make them "smaller", for example, by finding the bounding addresses of "interest" and trimming the block size. You could sort the blocks differently, perhaps prioritizing when a block points into itself, or add additional exit conditions. But this is plenty fast, even more so if you use threads for `prepare_optimized_scan` as well!

Another idea would be dropping some blocks entirely (although this is partially mitigated thanks to `Block::pointed_from`). If a base block (i.e. it starts where a module does) doesn't belong to the program in question (for example, it belongs to a system DLL), we could drop it, and don't even consider it in `prepare_optimized_scan`. This is probably what Cheat Engine is doing with "Include system modules", although I haven't experimented much with that option. The downside is, if it just so happens the offsets follow a path through that block, it won't be found. But it shouldn't be a big deal when plenty of paths are found.

In order to ignore system DLLs, it should be possible to find the module names and then where are they located (pretty much emulating the [Dynamic-Link Library Search Order][dllsearch]). If it falls within system directories, then we would ignore it.

If we want to reduce the search-space even more, we could specify a range of addresses. When any address falls outside this range, it is ignored. I believe Cheat Engine's default 0000000000000000-7FFFFFFFFFFFFFFF range is pretty much "scan all of it", as we're doing, but with more knowledge of the program at hand, you could definitely narrow this down.

Because we're not directly working with offsets (they are calculated after, and not before finding a candidate), I'm not sure how we could accurately implement Cheat Engine's option for "maximum offsets per node". Perhaps by building a temporary `HashSet`, sorting them in descending order, and only considering the first few smallest ones? More testing would be necessary to see if this is worthwhile. Beyond this last optimization, I can't think of any other worthwhile implementing though. We should be getting pretty close to somewhere optimal.

Anyway, let's finish this tutorial step, shall we?:

```rust
let addr = offset_list
    .iter()
    .take(offset_list.len() - 1)
    .fold(0, |base, offset| {
        usize::from_ne_bytes(
            process
                .read_memory(base + offset, 8)
                .unwrap()
                .try_into()
                .unwrap(),
        )
    })
    + offset_list.last().unwrap();

// Ta-dah!
process.write_memory(addr, 5000).unwrap();
```

## Retrospective

<span class="dim"><em>This section was added in a later edit.</em></span>

After letting this post settle down on me, I realized we probably managed to re-invent the way Cheat Engine works, or at least most of it, something I'm quite proud of! If you want to have this idea "click" in your head by yourself, you can skip this section. But really, there's an awful lot of similarities, and even matching terminology to some extent. Recall back in the [scan options](#scan-options) section, the two primary modes were *Scan for address* and *Generate pointermap*.

Scanning for an address with the setting "Compare results with other saved pointermap(s)" straight up sounds like the solution we came up with. We take two snapshots (the older one being equivalent to Cheat Engine's "saved pointermap") and perform a scan for the desired address, while comparing our intermediate results with the other pointermap to make sure it is still valid. Its job is to find all candidate paths, and if you were not comparing it to anything, obviously this would lead to a lot of false positives, which is why Cheat Engine advices against it.

Remember when we talked about "the pointerscanner goes through the list of pointervalues with a specific value"? This sounds a lot like our queue, too. The scan settings even mention "Static and dynamic queue sizes", possibly hinting at this implementation detail (as opposed to using unbounded recursion).

And what could a pointermap be other than… a mapping between pointers? This sounds like an awful lot to our "pre-scan" which scanned all the regions to find out "which regions could contain valid pointers into which other regions". That's a mapping of regions as determined by the pointers contained within them, and perhaps Cheat Engine only cares to store worthwhile snapshots of the memory and the corresponding regions. Maybe this is what Cheat Engine means by limiting the scan only to certain regions!



## Finale

And this my dear readers concludes my ambitions with the project! I think the program is pretty useful by now, even if it can only do a small fraction of what Cheat Engine can (I don't think I'm ready to write a form designer GUI yet… wait why was this part of Cheat Engine again?). ~~Despite the length of this entry, we didn't even figure out how Cheat Engine's pointer scanner works. Maybe it really is finding millions of possible paths, perhaps storing the offsets in some compact way~~. Although we can't know for sure what Cheat Engine is doing behind the scenes without studying its source code, we came pretty darn close to it. Let's recap what we do have learnt:

* We're experts in pointers by now! Seven layers of indirection? Easy peasy.
* There's a lot of configuration available for pointer scans: search depth, search breadth, search order, memory ranges, memory maps…
* One way to turn exponential problems into something more approachable is either finding an algorithm without the exponential growth, or trimming the amount of work to be done by *a lot*. And sometimes the former alternative is impossible.

The [code for this post][code] is available over at my GitHub. You can run `git checkout step8` after cloning the repository to get the right version of the code. If you're feeling up for a challenge, try to find a different, faster way (as in, less computationally-expensive) in which you can complete this tutorial step. Although ways to cut down the amount of work that needs to be done are definitely welcome, I'm looking for an entirely different approach, which can, for the most part, side-step the "there's too much work" issue.

In the next post, we'll tackle the ninth step of the tutorial: Shared code. I'm hoping it won't be too difficult, although there will be some learning that needs to be done. After that, I'll probably conclude the series. Maybe there could be some bonus episode in the future, or some other form of progress update. Until next time!

### Footnotes

[^1]: I spent a good chunk of time figuring out how to get this effect on the text (and borrowing code from several sites), but I'm extremely satisfied with the result. You do need a "modern" browser to see what I mean, though. I also lost it after the fact and had to redo it. Oh well.

[^2]: Actually, over a couple hundred are often found. But there's a high chance most of them would work just fine.

[^3]: I've gone through a lot of iterations for this post, with a fair amount of messy code, so this time I'll be explaining my thought process with new code rather than embedding what I've actually ended up writing.

[^4]: Only if `base` starts off as an aligned address, of course. But I think memory regions must start at multiples of the page size, which is a (relatively) large power of two, so it's safe to assume `base` is divisible by 8. You could throw in an `assert_eq!(base % 8, 0)` if you wanted to be extra sure.

[^5]: Although, just like we assume `base` is a multiple of 8, the `RegionSize` probably is as well.

[^6]: We could `mem::transmute` from `*const u8` to `*const usize` and dereference, but then we need to be careful about alignment, and `from_ne_bytes` seems to be plenty fast already.

[^7]: Not that we actually care about debug builds, as they run several orders of magnitude slower. But still, `wrapping_sub` has the right semantics here.

[^8]: Most of the time pointers point to the beginning of some structure, not its end, so accessing this structure's fields is done by adding, and not substracting, an offset from the pointer-value. For example:

```rust
#[repr(C)]
struct Vector {
    x: i32,
    y: i32,
}

let vec = Vector { x: 1, y: 1 };

let vec_ref = &vec;
let y_ref = &vec_ref.y;

let vec_ptr_val = vec_ref as *const _ as usize;
let y_ptr_val = y_ref as *const _ as usize;

assert_eq!(vec_ptr_val + 4, y_ptr_val);
```

[^9]: Or the top-depth, however you want to see it. I personally prefer starting at the highest depth so that when zero is reached, we know we're at the end.

[^10]: Which really, I don't think is worth it at all. If Cheat Engine is finding millions of *entire paths*, what kind of magic is it using to find cycles at any two depths???

[^11]: Yes, `Process::read_memory` could be changed to take in a buffer as input instead, so that it can be reused. Or it could even have an internal buffer. But we won't be using this method much anyway.

[^12]: I prefer starting on the second snapshot because it feels more "fresh", as it's the latest one, although it doesn't really matter, because the path we're looking for must be valid in both anyway.

[^13]: Maybe the recursive `run` could run in a pool of threads?

[^14]: Most heaps tend to be min-heaps, and it's not uncommon for the use of `BinaryHeap` in Rust to need `std::cmp::Reverse` in order to get [min-heap behaviour][minheap]. There's been some discussion on internals about this, such as [Why is std::collections::BinaryHeap a max-heap?][whymaxheap] and more recently [Specializing BinaryHeap to MaxHeap and MinHeap][maxtominheap] where @matklad laments:

> I feel like our heap accumulated a bunch of problems (wrong default order, slow into-sorted, wrong into-iter, confusing naming, slow-perf due to being binary).

[^15]: This sounds like it would be most useful when you've already put the work before, and is now time for a re-scan. In this scenario, you already know that there's probably some golden "offset" into the structure you care about.

[^16]: 87.5% for us, thanks to having 8-byte sized pointers!

[semsat]: https://en.wikipedia.org/wiki/Semantic_satiation
[`crossbeam-utils`]: https://crates.io/crates/crossbeam-utils
[dllsearch]: https://docs.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order
[minheap]: https://doc.rust-lang.org/stable/std/collections/struct.BinaryHeap.html#min-heap
[whymaxheap]: https://internals.rust-lang.org/t/why-is-std-binaryheap-a-max-heap/11498
[maxtominheap]: https://internals.rust-lang.org/t/specializing-binaryheap-to-maxheap-and-minheap/15115
[code]: https://github.com/lonami/memo
