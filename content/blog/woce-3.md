+++
title = "Writing our own Cheat Engine: Unknown initial value"
date = 2021-02-19
updated = 2021-02-19
[taxonomies]
category = ["sw"]
tags = ["windows", "rust", "hacking"]
+++

This is part 3 on the *Writing our own Cheat Engine* series:

* [Part 1: Introduction](/blog/woce-1) (start here if you're new to the series!)
* [Part 2: Exact Value scanning](/blog/woce-2)
* Part 3: Unknown initial value

In part 2 we left off with a bit of a cliff-hanger. Our little program is now able to scan for an exact value, remember the couple hundred addresses pointing to said value, and perform subsequent scans to narrow the list of addresses down until we're left with a handful of them.

However, it is not always the case that you have an exact value to work with. The best you can do in these cases is guess what the software might be storing. For example, it could be a floating point for your current movement speed in a game, or an integer for your current health.

The problem with this is that there are far too many possible locations storing our desired value. If you count misaligned locations, this means there is a different location to address every single byte in memory. A program with one megabyte of memory already has a *million* of addresses. Clearly, we need to do better than performing one million memory reads[^1].

This post will shift focus a bit from using `winapi` to possible techniques to perform the various scans.

## Unknown initial value

<details open><summary>Cheat Engine Tutorial: Step 3</summary>

> Ok, seeing that you've figured out how to find a value using exact value let's move on to the next step.
>
> First things first though. Since you are doing a new scan, you have to click on New Scan first, to start a new scan. (You may think this is straighforward, but you'd be surprised how many people get stuck on that step) I won't be explaining this step again, so keep this in mind
> Now that you've started a new scan, let's continue
>
> In the previous test we knew the initial value so we could do a exact value, but now we have a status bar where we don't know the starting value.
> We only know that the value is between 0 and 500. And each time you click 'hit me' you lose some health. The amount you lose each time is shown above the status bar.
>
> Again there are several different ways to find the value. (like doing a decreased value by... scan), but I'll only explain the easiest. "Unknown initial value", and decreased value.
> Because you don't know the value it is right now, a exact value wont do any good, so choose as scantype 'Unknown initial value', again, the value type is 4-bytes. (most windows apps use 4-bytes)click first scan and wait till it's done.
>
> When it is done click 'hit me'. You'll lose some of your health. (the amount you lost shows for a few seconds and then disappears, but you don't need that)
> Now go to Cheat Engine, and choose 'Decreased Value' and click 'Next Scan'
> When that scan is done, click hit me again, and repeat the above till you only find a few.
>
> We know the value is between 0 and 500, so pick the one that is most likely the address we need, and add it to the list.
> Now change the health to 5000, to proceed to the next step.

</details>

## Dense memory locations

The key thing to notice here is that, when we read memory from another process, we do so over *entire regions*. A memory region is represented by a starting offset, a size, and a bunch of other things like protection level.

When running the first scan for an unknown value, all we need to remember is the starting offset and size for every single region. All the candidate locations that could point to our value fall within this range, so it is enough for us to store the range definition, and not every location within it.

To gain a better understanding of what this means, let's come up with a more specific scenario. With our current approach of doing things, we store an address (`usize`) for every location pointing to our desired value. In the case of unknown values, all locations are equally valid, since we don't know what value they should point to yet, and any value they point to is good. With this representation, we would end up with a very large vector:

```rust
let locations = vec![0x2000, 0x2001, ..., 0x20ff, 0x2100];
```

This representation is dense. Every single number in the range `0x2000..=0x2100` is present. So why bother storing the values individually when the range is enough?:

```rust
let locations = EntireRegion { range: 0x2000..=0x2100 };
```

Much better! With two `usize`, one for the starting location and another for the end, we can indicate that we care about all the locations falling in that range.

In fact, some accessible memory regions immediately follow eachother, so we could even compact this further and merge regions which are together. But due to their potential differences with regards to protection levels, we will not attempt to merge regions.

We don't want to get rid of the old way of storing locations, because once we start narrowing them down, we will want to go back to storing just a few candidates. To keep things tidy, let's introduce a new `enum` representing either possibility:

```rust
use std::ops::Range;

pub enum CandidateLocations {
    Discrete {
        locations: Vec<usize>,
    },
    Dense {
        range: Range<usize>,
    }
}
```

Let's also introduce another `enum` to perform the different scan types. For the time being, we will only worry about looking for `i32` in memory:

```rust
pub enum Scan {
    Exact(i32),
    Unknown,
}
```

## Storing scanned values

When scanning for exact values, it's not necessary to store the value found. We already know they're all the same, for example, value `42`. However, if the value is unknown, we do need to store it so that we can compare it in a subsequent scan to see if the value is the same or it changed. This means the value can be "any within" the read memory chunk:

```rust
pub enum Value {
    Exact(i32),
    AnyWithin(Vec<u8>),
}
```

For every region in memory, there will be some candidate locations and a value (or value range) we need to compare against in subsequent scans:

```rust
pub struct Region {
    pub info: winapi::um::winnt::MEMORY_BASIC_INFORMATION,
    pub locations: CandidateLocations,
    pub value: Value,
}
```

With all the data structures needed setup, we can finally refactor our old scanning code into a new method capable of dealing with all these cases. For brevity, I will omit the exact scan, as it remains mostly unchanged:

```rust
use winapi::um::winnt::MEMORY_BASIC_INFORMATION;

...

// inside `impl Process`
pub fn scan_regions(&self, regions: &[MEMORY_BASIC_INFORMATION], scan: Scan) -> Vec<Region> {
    regions
        .iter()
        .flat_map(|region| match scan {
            Scan::Exact(n) => todo!("old scan implementation"),
            Scan::Unknown => {
                let base = region.BaseAddress as usize;
                match self.read_memory(region.BaseAddress as _, region.RegionSize) {
                    Ok(memory) => Some(Region {
                        info: region.clone(),
                        locations: CandidateLocations::Dense {
                            range: base..base + region.RegionSize,
                        },
                        value: Value::AnyWithin(memory),
                    }),
                    Err(_) => None,
                }
            }
        })
        .collect()
}
```

Time to try it out!

```rust
impl CandidateLocations {
    pub fn len(&self) -> usize {
        match self {
            CandidateLocations::Discrete { locations } => locations.len(),
            CandidateLocations::Dense { range } => range.len(),
        }
    }
}

...

fn main() {
    // -snip-

    println!("Scanning {} memory regions", regions.len());
    let last_scan = process.scan_regions(&regions, Scan::Unknown);
    println!(
        "Found {} locations",
        last_scan.iter().map(|r| r.locations.len()).sum::<usize>()
    );
}
```

```
Scanning 88 memory regions
Found 3014656 locations
```

If we consider misaligned locations, there is a lot of potential addresses where we could look for. Running the same scan on Cheat Engine yields `2,449,408` addresses, which is pretty close. It's probably skipping some additional regions that we are considering. Emulating Cheat Engine to perfection is not a concern for us at the moment, so I'm not going to investigate what regions it actually uses.

## Comparing scanned values

Now that we have performed the initial scan and have stored all the `CandidateLocations` and `Value`, we can re-implement the "next scan" step to handle any variant of our `Scan` enum. This enables us to mix-and-match any `Scan` mode in any order. For example, one could perform an exact scan, then one for decreased values, or start with unknown scan and scan for unchanged values.

The tutorial suggests using "decreased value" scan, so let's start with that:

```rust
pub enum Scan {
    Exact(i32),
    Unknown,
    Decreased, // new!
}
```

Other scanning modes, such as decreased by a known amount rather than any decrease, increased, unchanged, changed and so on, are not very different from the "decreased" scan, so I won't bore you with the details.

I will use a different method to perform a "rescan", since the first one is a bit more special in that it doesn't start with any previous values:

```rust
pub fn rescan_regions(&self, regions: &[Region], scan: Scan) -> Vec<Region> {
    regions
        .iter()
        .flat_map(|region| match scan {
            Scan::Decreased => {
                let mut locations = Vec::new();
                match region.locations {
                    CandidateLocations::Dense { range } => {
                        match self.read_memory(range.start, range.end - range.start) {
                            Ok(memory) => match region.value {
                                Value::AnyWithin(previous) => {
                                    memory
                                        .windows(4)
                                        .zip(previous.windows(4))
                                        .enumerate()
                                        .step_by(4)
                                        .for_each(|(offset, (new, old))| {
                                            let new = i32::from_ne_bytes([
                                                new[0], new[1], new[2], new[3],
                                            ]);
                                            let old = i32::from_ne_bytes([
                                                old[0], old[1], old[2], old[3],
                                            ]);
                                            if new < old {
                                                locations.push(range.start + offset);
                                            }
                                        });

                                    Some(Region {
                                        info: region.info.clone(),
                                        locations: CandidateLocations::Discrete { locations },
                                        value: Value::AnyWithin(memory),
                                    })
                                }
                                _ => todo!(),
                            },
                            _ => todo!(),
                        }
                    }
                    _ => todo!(),
                }
            }
            _ => todo!(),
        })
        .collect()
}
```

If you've skimmed over that, I do not blame you. Here's the summary: for every existing region, when executing the scan mode "decreased", if the previous locations were dense, read the entire memory region. On success, if the previous values were a chunk of memory, iterate over the current and old memory at the same time, and for every aligned `i32`, if the new value is less, store it.

It's also making me ill. Before I leave a mess on the floor, does it work?

```rust
std::thread::sleep(std::time::Duration::from_secs(10));
let last_scan = process.rescan_regions(&last_scan, Scan::Decreased);
println!(
    "Found {} locations",
    last_scan.iter().map(|r| r.locations.len()).sum::<usize>()
);
```

```rust
Found 3014656 locations
Found 177 locations
```

Okay, great, let's clean up this mess…

## Refactoring

Does it also make you uncomfortable to be writing something that you know will end up *huge* unless you begin refactoring other parts right now? I definitely feel that way. But I think it's good discipline to push through with something that works first, even if it's nasty, before going on a tangent. Now that we have the basic implementation working, let's take on this monster before it eats us alive.

First things first, that method is inside an `impl` block. The deepest nesting level is 13. I almost have to turn around my chair to read the entire thing out!

Second, we're nesting four matches. Three of them we care about: scan, candidate location, and value. If each of these `enum` has `S`, `C` and `V` variants respectively, writing each of these by hand will require `S * C * V` different implementations! Cheat Engine offers 10 different scans, I can think of at least 3 different ways to store candidate locations, and another 3 ways to store the values found. That's `10 * 3 * 3 = 90` different combinations. I am not willing to write out all these[^2], so we need to start introducing some abstractions. Just imagine what a monster function you would end with! The horror!

Third, why is the scan being executed in the process? This is something that should be done in the `impl Scan` instead!

Let's begin the cleanup:

```rust
pub fn rescan_regions(&self, regions: &[Region], scan: Scan) -> Vec<Region> {
    todo!()
}
```

I already feel ten times better.

Now, this method will unconditionally read the entire memory region, even if the scan or the previous candidate locations don't need it[^3]. In the worst case with a single discrete candidate location, we will be reading a very large chunk of memory when we could have read just the 4 bytes needed for the `i32`. On the bright side, if there *are* more locations in this memory region, we will get read of them at the same time[^4]. So even if we're moving more memory around all the time, it isn't *too* bad.

```rust
regions
    .iter()
    .flat_map(
        |region| match self.read_memory(region.info.BaseAddress as _, region.info.RegionSize) {
            Ok(memory) => todo!(),
            Err(err) => {
                eprintln!(
                    "Failed to read {} bytes at {:?}: {}",
                    region.info.RegionSize, region.info.BaseAddress, err,
                );
                None
            }
        },
    )
    .collect()
```

Great! If reading memory succeeds, we want to rerun the scan:

```rust
Ok(memory) => Some(scan.rerun(region, memory)),
```

The rerun will live inside `impl Scan`:

```rust
pub fn rerun(&self, region: &Region, memory: Vec<u8>) -> Region {
    match self {
        Scan::Exact(_) => self.run(region.info.clone(), memory),
        Scan::Unknown => region.clone(),
        Scan::Decreased => todo!(),
    }
}
```

An exact scan doesn't care about any previous values, so it behaves like a first scan. The first scan is done by the `run` function (it contains the implementation factored out of the `Process::scan_regions` method), which only needs the region information and the current memory chunk we just read.

The unknown scan leaves the region unchanged: any value stored is still valid, because it is unknown what we're looking for.

The decreased scan will have to iterate over all the candidate locations, and compare them with the current memory chunk. But this time, we'll abstract this iteration too:

```rust
impl Region {
    fn iter_locations<'a>(
        &'a self,
        new_memory: &'a [u8],
    ) -> impl Iterator<Item = (usize, i32, i32)> + 'a {
        match &self.locations {
            CandidateLocations::Dense { range } => range.clone().step_by(4).map(move |addr| {
                let old = self.value_at(addr);
                let new = i32::from_ne_bytes([
                    new_memory[0],
                    new_memory[1],
                    new_memory[2],
                    new_memory[3],
                ]);
                (addr, old, new)
            }),
            _ => todo!(),
        }
    }
}
```

For a dense candidate location, we iterate over all the 4-aligned addresses (fast scan for `i32` values), and yield `(current address, old value, new value)`. This way, the `Scan` can do anything it wants with the old and new values, and if it finds a match, it can use the address.

The `value_at` method will deal with all the `Value` variants:

```rust
fn value_at(&self, addr: usize) -> i32 {
    match &self.value {
        Value::AnyWithin(chunk) => {
            let base = addr - self.info.BaseAddress as usize;
            let bytes = &chunk[base..base + 4];
            i32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
        }
        _ => todo!(),
    }
}
```

This way, `iter_locations` can easily use any value type. With this, we have all `enum` covered: `Scan` in `rerun`, `CandidateLocation` in `iter_locations`, and `Value` in `value_at`. Now we can add as many variants as we want, and we will only need to update a single `match` arm for each of them. Let's implement `Scan::Decreased` and try it out:

```rust
pub fn rerun(&self, region: &Region, memory: Vec<u8>) -> Region {
    match self {
        Scan::Decreased => Region {
            info: region.info.clone(),
            locations: CandidateLocations::Discrete {
                locations: region
                    .iter_locations(&memory)
                    .flat_map(|(addr, old, new)| if new < old { Some(addr) } else { None })
                    .collect(),
            },
            value: Value::AnyWithin(memory),
        },,
    }
}
```

```
Found 3014656 locations
Found 223791 locations
```

Hmm… before we went down from `3014656` to `177` locations, and now we went down to `223791`. Where did we go wrong?

After spending several hours on this, I can tell you where we went wrong. `iter_locations` is always accessing the memory range `0..4`, and not the right address. Here's the fix:

```rust
CandidateLocations::Dense { range } => range.clone().step_by(4).map(move |addr| {
    let old = self.value_at(addr);
    let base = addr - self.info.BaseAddress as usize;
    let bytes = &new_memory[base..base + 4];
    let new = i32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    (addr, old, new)
}),
```

## Going beyond

Let's take a look at other possible `Scan` types. Cheat Engine supports the following initial scan types:

* Exact Value
* Bigger than…
* Smaller than…
* Value between…
* Unknown initial value

"Bigger than" and "Smaller than" can both be represented by "Value between", so it's pretty much just three.

For subsequent scans, in addition to the scan types described above, we find:

* Increased value
* Increased value by…
* Decreased value
* Decreased value by…
* Changed value
* Unchanged value

Not only does Cheat Engine provide all of these scans, but all of them can also be negated. For example, "find values that were not increased by 7". One could imagine to also support things like "increased value by range". For the increased and decreased scans, Cheat Engine also supports "at least xx%", so that if the value changed within the specified percentage interval, it will be considered.

What about `CandidateLocations`? I can't tell you how Cheat Engine stores these, but I can tell you that `CandidateLocations::Discrete` can still be quite inefficient. Imagine you've started with a scan for unknown values and then ran a scan for unchanged valueus. Most values in memory will have been unchanged, but with our current implementation, we are now storing an entire `usize` address for each of these. One option would be to introduce `CandidateLocations::Sparse`, which would be a middle ground. You could implement it like `Dense` and include a vector of booleans telling you which values to consider, or go smaller and use a bitstring or bit vector. You could use a sparse vector data structure.

`Value` is very much like `CandidateLocations`, except that it stores a value to compare against and not an address. Here we can either have an exact value, or an older copy of the memory. Again, keeping a copy of the entire memory chunk when all we need is a handful of values is inefficient. You could keep a mapping from addresses to values if you don't have too many. Or you could shrink and fragment the copied memory in a more optimal way. There's a lot of room for improvement!

What if, despite all of the efforts above, we still don't have enough RAM to store all this information? The Cheat Engine Tutorial doesn't use a lot of memory, but as soon as you try scanning bigger programs, like games, you may find yourself needing several gigabytes worth of memory to remember all the found values in order to compare them in subsequent scans. You may even need to consider dumping all the regions to a file and read from it to run the comparisons. For example, running a scan for "unknown value" in Cheat Engine brings its memory up by the same amount of memory used by the process scanned (which makes sense), but as soon as I ran a scan for "unchanged value" over the misaligned values, Cheat Engine's disk usage skyrocketed to 1GB/s (!) for several seconds on my SSD. After it finished, memory usage went down to normal. It was very likely writing out all candidate locations to disk.

## Finale

There is a lot of things to learn from Cheat Engine just by observing its behaviour, and we're only scratching its surface.

In the next post, we'll tackle the fourth step of the tutorial: Floating points. So far, we have only been working with `i32` for simplicity. We will need to update our code to be able to account for different data types, which will make it easy to support other types like `i16`, `i64`, or even strings, represented as an arbitrary sequence of bytes.

As usual, you can [obtain the code for this post][code] over at my GitHub. You can run `git checkout step3` after cloning the repository to get the right version of the code. This version is a bit cleaner than the one presented in the blog, and contains some of the things described in the [Going beyond](#going-beyond) section. Until next time!

### Footnotes

[^1]: Well, technically, we will perform a million memory reads[^5]. The issue here is the million calls to `ReadProcessMemory`, not reading memory per se.

[^2]: Not currently. After a basic implementation works, writing each implementation by hand and fine-tuning them by treating each of them as a special case could yield significant speed improvements. So although it would be a lot of work, this option shouldn't be ruled out completely.

[^3]: You could ask the candidate locations where one should read, which would still keep the code reasonably simple.

[^4]: You could also optimize for this case by determining both the smallest and largest address, and reading enough to cover them both. Or apply additional heuristics to only do so if the ratio of the size you're reading compared to the size you need isn't too large and abort the joint read otherwise. There is a lot of room for optimization here.

[^5]: (A footnote in a footnote?) The machine registers, memory cache and compiler will all help lower this cost, so the generated executable might not actually need that many reads from RAM. But that's getting way too deep into the details now.

[script-kid]: https://www.urbandictionary.com/define.php?term=script%20kiddie
[code]: https://github.com/lonami/memo
