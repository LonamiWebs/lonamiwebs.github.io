+++
title = "Writing our own Cheat Engine: Floating points"
date = 2021-02-28
updated = 2021-02-28
[taxonomies]
category = ["sw"]
tags = ["windows", "rust", "hacking"]
+++

This is part 4 on the *Writing our own Cheat Engine* series:

* [Part 1: Introduction](/blog/woce-1) (start here if you're new to the series!)
* [Part 2: Exact Value scanning](/blog/woce-2)
* [Part 3: Unknown initial value](/blog/woce-3)
* Part 4: Floating points
* [Part 5: Code finder](/blog/woce-5)
* [Part 6: Pointers](/blog/woce-6)

In part 3 we did a fair amount of plumbing in order to support scan modes beyond the trivial "exact value scan". As a result, we have abstracted away the `Scan`, `CandidateLocations` and `Value` types as a separate `enum` each. Scanning for changed memory regions in an opened process can now be achieved with three lines of code:

```rust
let regions = process.memory_regions();
let first_scan = process.scan_regions(&regions, Scan::InRange(0, 500));
let second_scan = process.rescan_regions(&first_scan, Scan::DecreasedBy(7));
```

How's that for programmability? No need to fire up Cheat Engine's GUI anymore!

The `first_scan` in the example above remembers all the found `Value` within the range specified by `Scan`. Up until now, we have only worked with `i32`, so that's the type the scans expect and what they work with.

Now it's time to introduce support for different types, like `f32`, `i64`, or even more atypical ones, like arbitrary sequences of bytes (think of strings) or even numbers in big-endian.

Tighten your belt, because this post is quite the ride. Let's get right into it!

## Floating points

<details open><summary>Cheat Engine Tutorial: Step 4</summary>

> In the previous tutorial we used bytes to scan, but some games store information in so called 'floating point' notations.
> (probably to prevent simple memory scanners from finding it the easy way). A floating point is a value with some digits behind the point. (like 5.12 or 11321.1)
>
> Below you see your health and ammo. Both are stored as Floating point notations, but health is stored as a float and ammo is stored as a double.
> Click on hit me to lose some health, and on shoot to decrease your ammo with 0.5
>
> You have to set BOTH values to 5000 or higher to proceed.
>
> Exact value scan will work fine here, but you may want to experiment with other types too.
>
> Hint: It is recommended to disable "Fast Scan" for type double

</details>

## Generic values

The `Value` enumeration holds scanned values, and is currently hardcoded to store `i32`. The `Scan` type also holds a value, the value we want to scan for. Changing it to support other types is trivial:

```rust
pub enum Scan<T> {
    Exact(T),
    Unknown,
    Decreased,
    // ...other variants...
}

pub enum Value<T> {
    Exact(T),
    AnyWithin(Vec<u8>),
}
```

`AnyWithin` is the raw memory, and `T` can be interpreted from any sequence of bytes thanks to our friend [`mem::transmute`][transmute]. This change alone is enough to store an arbitrary `T`! So we're done now? Not really, no.

First of all, we need to update all the places where `Scan` or `Value` are used. Our first stop is the scanned `Region`, which holds the found `Value`:

```rust
pub struct Region<T> {
    pub info: MEMORY_BASIC_INFORMATION,
    pub locations: CandidateLocations,
    pub value: Value<T>,
}
```

Then, we need to update everywhere `Region` is used, and on and on… All in all this process is just repeating `cargo check`, letting the compiler vent on you, and taking good care of it by fixing the errors. It's quite reassuring to know you will not miss a single place. Thank you, compiler!

But wait, how could scanning for a decreased value work for any `T`? The type is not `Ord`, we should add some trait bounds. And also, what happens if the type is not `Copy`? It could implement `Drop`[^1], and we will be transmuting from raw bytes, which would trigger the `Drop` implementation when we're done with the value! Not memory safe at all! And how could we possibly cast raw memory to the type without knowing its siz– oh nevermind, [`T` is already `Sized` by default][sized-default]. But anyway, we need the other bounds.

In order to not repeat ourselves, we will implement a new `trait`, let's say `Scannable`, which requires all other bounds:

```rust
pub trait Scannable: Copy + PartialEq + PartialOrd {}

impl<T: Copy + PartialEq + PartialOrd> Scannable for T {}
```

And fix our definitions:

```rust
pub enum Scan<T: Scannable> { ... }
pub enum Value<T: Scannable> { ... }
pub struct Region<T: Scannable> { ... }

// ...and the many other places referring to T
```

Every type which is `Copy`, `PartialEq` and `PartialOrd` can be scanned over[^2], because we `impl Scan for T` where the bounds are met. Unfortunately, we cannot require `Eq` or `Ord` because the floating point types do not implement it.

## Transmuting memory

Also known as reinterpreting a bunch of bytes as something else, or perhaps it stands for "summoning the demon":

> `transmute` is **incredibly** unsafe. There are a vast number of ways to cause [undefined behavior][ub] with this function. `transmute` should be the absolute last resort.

Types like `i32` define methods such as [`from_ne_bytes`][fromne] and [`to_ne_bytes`][tone] which convert raw bytes from and into its native representation. This is all really nice, but unfortunately, there's no standard trait in the Rust's standard library to "interpret a type `T` as the byte sequence of its native representation". `transmute`, however, does exist, and similar to any other `unsafe` function, it's safe to call **as long as we respect its invariants**. What are these invariants[^3]?

> Both types must have the same size

Okay, we can just assert that the window length matches the type's length. What else?

> Neither the original, nor the result, may be an [invalid value][inv-val].

What's an invalid value?

> * a `bool` that isn't 0 or 1
> * an `enum` with an invalid discriminant
> * a null `fn` pointer
> * a `char` outside the ranges [0x0, 0xD7FF] and [0xE000, 0x10FFFF]
> * a `!` (all values are invalid for this type)
> * an integer (`i*`/`u*`), floating point value (`f*`), or raw pointer read from uninitialized memory, or uninitialized memory in a `str`.
> * a reference/`Box` that is dangling, unaligned, or points to an invalid value.
> * a wide reference, `Box`, or raw pointer that has invalid metadata:
>   * `dyn Trait` metadata is invalid if it is not a pointer to a vtable for `Trait` that matches the actual dynamic trait the pointer or reference points to
>   * slice metadata is invalid if the length is not a valid `usize` (i.e., it must not be read from uninitialized memory)
> * a type with custom invalid values that is one of those values, such as a `NonNull` that is null. (Requesting custom invalid values is an unstable feature, but some stable libstd types, like `NonNull`, make use of it.)

Okay, that's actually an awful lot. Types like `bool` implement all the trait bounds we defined, and it would be insta-UB to ever try to cast them from arbitrary bytes. The same goes for `char`, and all `enum` are out of our control, too. At least we're safe on the "memory is initialized" front.

Dang it, I really wanted to use `transmute`! But if we were to use it for arbitrary types, it would trigger undefined behaviour sooner than later.

We have several options here:

* Make it an `unsafe trait`. Implementors will be responsible for ensuring that the type they're implementing it for can be safely transmuted from and into.
* [Seal the `trait`][seal] and implement it only for types we know are safe[^4], like `i32`.
* Add methods to the `trait` definition that do the conversion of the type into its native representation.

We will go with the first option[^5], because I really want to use `transmute`, and I want users to be able to implement the trait on their own types.

In any case, we need to change our `impl` to something more specific, in order to prevent it from automatically implementing the trait for types for which their memory representation has invalid values. So we get rid of this:

```rust
pub trait Scannable: Copy + PartialEq + PartialOrd {}

impl<T: Copy + PartialEq + PartialOrd> Scannable for T {}
```

And replace it with this:

```rust
pub unsafe trait Scannable: Copy + PartialEq + PartialOrd {}

macro_rules! impl_many {
    ( unsafe impl $trait:tt for $( $ty:ty ),* ) => {
        $( unsafe impl $trait for $ty {} )*
    };
}

// SAFETY: all these types respect `Scannable` invariants.
impl_many!(unsafe impl Scannable for i8, u8, i16, u16, i32, u32, i64, u64, f32, f64);
```

Making a small macro for things like these is super useful. You could of course write `unsafe impl Scannable for T` for all ten `T` as well, but that introduces even more `unsafe` to read. Last but not least, let's replace the hardcoded `i32::from_ne_bytes` and `i32::to_ne_bytes` with `mem::transmute`.

All the `windows(4)` need to be replaced with `windows(mem::size_of::<T>())` because the size may no longer be `4`. All the `i32::from_ne_bytes(...)` need to be replaced with `mem::transmute::<_, T>(...)`. We explicitly write out `T` to make sure the compiler doesn't accidentally infer something we didn't intend.

And… it doesn't work at all. We're working with byte slices of arbitrary length. We cannot transmute a `&[]` type, which is 16 bytes (8 for the pointer and 8 for the length), to our `T`. My plan to use transmute can't possibly work here. Sigh.

## Not quite transmuting memory

Okay, we can't transmute, because we don't have a sized value, we only have a slice of bytes pointing somewhere else. What we *could* do is reinterpret the pointer to those bytes as a different type, and then dereference it! This is still a form of "transmutation", just without using `transmute`.

```rust
let value = unsafe { *(window.as_ptr() as *const T) };
```

Woop! You can compile this and test it out on the step 2 and 3 of the tutorial, using `i32`, and it will still work! Something troubles me, though. Can you see what it is?

When we talked about invalid values, it had a note about unaligned references:

> a reference/`Box` that is dangling, unaligned, or points to an invalid value.

Our `window` is essentially a reference to `T`. The only difference is we're working at the pointer level, but they're pretty much references. Let's see what the documentation for [`pointer`][pointer] has to say as well, since we're dereferencing pointers:

> when a raw pointer is dereferenced (using the `*` operator), it must be non-null and aligned.

It must be aligned. The only reason why our data is aligned is because we are also performing a "fast scan", so we only look at aligned locations. This is a time bomb waiting to blow up. Is there any other way to [`read`][ptr-read] from a pointer which is safer?

> `src` must be properly aligned. Use [`read_unaligned`][ptr-readun] if this is not the case.

Bingo! Both `read` and `read_unaligned`, unlike dereferencing the pointer, will perform a copy, but if it can make the code less prone to blowing up, I'll take it[^6]. Let's change the code one more time:

```rust
let current = unsafe { window.as_ptr().cast::<T>().read_unaligned() };
```

I prefer to avoid type annotations in variables where possible, which is why I use the [turbofish] so often. You can get rid of the cast and use a type annotation instead, but make sure the type is known, otherwise it will think it's `u8` because `window` is a `&[u8]`.

Now, this is all cool and good. You can replace `i32` with `f32` for `T` and you'll be able to get halfway done with the step 4 of Cheat Engine's tutorial. Unfortunately, as it is, this code is not enough to complete step 4 with exact scans[^7]. You see, comparing floating point values is not as simple as checking for bitwise equality. We were actually really lucky that the `f32` part works! But the values in the `f64` part are not as precise as our inputs, so our exact scan fails.

Using a fixed type parameter is pretty limiting as well. On the one hand, it is nice that, if you scan for `i32`, the compiler statically guarantees that subsequent scans will also happen on `i32` and thus be compatible. On the other, this requires us to know the type at compile time, which for an interactive program, is not possible. While we *could* create different methods for each supported type and, at runtime, decide to which we should jump, I am not satisfied with that solution. It also means we can't switch from scanning an `u32` to an `i32`, for whatever reason.

So we need to work around this once more.

## Rethinking the scans

What does our scanning function need, really? It needs a way to compare two chunks of memory as being equal or not (as we have seen, this isn't trivial with types such as floating point numbers) and, for other types of scans, it needs to be able to produce an ordering, or calculate a difference.

Instead of having a our trait require the bounds `PartialEq` and `PartialOrd`, we can define our own methods to compare `Self` with `&[u8]`. It still should be `Clone`, so we can pass it around without worrying about lifetimes:

```rust
// Callers must `assert_eq!(memory.len(), mem::size_of::<Self>())`.
unsafe fn eq(&self, memory: &[u8]) -> bool;
unsafe fn cmp(&self, memory: &[u8]) -> Ordering;
```

This can be trivially implemented for all integer types:

```rust
macro_rules! impl_scannable_for_int {
    ( $( $ty:ty ),* ) => {
        $(
            // SAFETY: caller is responsible to `assert_eq!(memory.len(), mem::size_of::<T>())`
            impl Scannable for $ty {
                unsafe fn eq(&self, memory: &[u8]) -> bool {
                    let other = unsafe { memory.as_ptr().cast::<$ty>().read_unaligned() };
                    *self == other
                }

                unsafe fn cmp(&self, memory: &[u8]) -> Ordering {
                    let other = unsafe { memory.as_ptr().cast::<$ty>().read_unaligned() };
                    <$ty as Ord>::cmp(self, &other)
                }
            }
        )*
    };
}

impl_scannable_for_int!(i8, u8, i16, u16, i32, u32, i64, u64);
```

The funny `<$ty as Ord>` is because I decided to call the method `Scannable::cmp`, so I have to disambiguate between it and `Ord::cmp`. We can go ahead and update the code using `Scannable` to use these new functions instead.

Now, you may have noticed I only implemented it for the integer types. That's because floats need some extra care. Unfortunately, floating point types do not have any form of "precision" embedded in them, so we can't accurately say "compare these floats to the precision level the user specified". What we can do, however, is drop a few bits from the mantissa, so "relatively close" quantities are considered equal. It's definitely not as good as comparing floats to the user's precision, but it will get the job done.

I'm going to arbitrarily say that we are okay comparing with "half" the precision. We can achieve that by masking half of the bits from the mantissa to zero:

```rust

macro_rules! impl_scannable_for_float {
    ( $( $ty:ty : $int_ty:ty ),* ) => {
        $(
            #[allow(unused_unsafe)] // mind you, it is necessary
            impl Scannable for $ty {
                unsafe fn eq(&self, memory: &[u8]) -> bool {
                    const MASK: $int_ty = !((1 << (<$ty>::MANTISSA_DIGITS / 2)) - 1);

                    // SAFETY: caller is responsible to `assert_eq!(memory.len(), mem::size_of::<T>())`
                    let other = unsafe { memory.as_ptr().cast::<$ty>().read_unaligned() };
                    let left = <$ty>::from_bits(self.to_bits() & MASK);
                    let right = <$ty>::from_bits(other.to_bits() & MASK);
                    left == right
                }

                ...
            }
        )*
    };
}

impl_scannable_for_float!(f32: u32, f64: u64);
```

You may be wondering what's up with that weird `MASK`. Let's visualize it with a [`f16`][f16]. This type has 16 bits, 1 for sign, 5 for exponent, and 10 for the mantissa:

```
S EEEEE MMMMMMMMMM
```

If we substitute the constant with the numeric value and operate:

```rust
!((1 << (10 / 2)) - 1)
!((1 << 5) - 1)
!(0b00000000_00100000 - 1)
!(0b00000000_00011111)
0b11111111_11100000
```

So effectively, half of the mantisssa bit will be masked to 0. For the `f16` example, this makes us lose 5 bits of precision. Comparing two floating point values with their last five bits truncated is equivalent to checking if they are "roughly equal"!

When Cheat Engine scans for floating point values, several additional settings show, and one such option is "truncated". I do not know if it behaves like this, but it might.

Let's try this out:

```rust
#[test]
fn f32_roughly_eq() {
    let left = 0.25f32;
    let right = 0.25000123f32;
    let memory = unsafe { mem::transmute::<_, [u8; 4]>(right) };
    assert_ne!(left, right);
    assert!(unsafe { Scannable::eq(&left, &memory) });
}
```

```
>cargo test f32_roughly_eq

running 1 test
test scan::candidate_location_tests::f32_roughly_eq ... ok
```

Huzzah! The `assert_ne!` makes sure that a normal comparision would fail, and then we `assert!` that our custom one passes the test. When the user performs an exact scan, the code will be more tolerant to the user's less precise inputs, which overall should result in a nicer experience.

## Dynamically sized scans

The second problem we need to solve is the possibility of the size not being known at compile time[^8]. While we can go as far as scanning over strings of a known length, this is rather limiting, because we need to know the length at compile time[^9]. Heap allocated objects are another problem, because we don't want to compare the memory representation of the stack object, but likely the memory where they point to (such as `String`).

Instead of using `mem::size_of`, we can add a new method to our `Scannable`, `size`, which will tell us the size required of the memory view we're comparing against:

```rust
unsafe impl Scannable {
    ...

    fn size(&self) -> usize;
}
```

It is `unsafe` to implement, because we are relying on the returned value to be truthful and unchanging. It should be safe to call, because it cannot have any invariants. Unfortunately, signaling "unsafe to implement" is done by marking the entire trait as `unsafe`, since "unsafe to call" is reserved for `unsafe fn`, and even though the rest of methods are not necessarily unsafe to implement, they're treated as such.

At the moment, `Scannable` cannot be made into a trait object because it is [not object safe][objectsafe]. This is caused by the `Clone` requirement on all `Scannable` object, which in turn needs the types to be `Sized` because `clone` returns `Self`. Because of this, the size must be known.

However, we *can* move the `Clone` requirement to the methods that need it! This way, `Scannable` can remain object safe, enabling us to do the following:

```rust
unsafe impl<T: AsRef<dyn Scannable> + AsMut<dyn Scannable>> Scannable for T {
    unsafe fn eq(&self, memory: &[u8]) -> bool {
        self.as_ref().eq(memory)
    }

    unsafe fn cmp(&self, memory: &[u8]) -> Ordering {
        self.as_ref().cmp(memory)
    }

    fn mem_view(&self) -> &[u8] {
        self.as_ref().mem_view()
    }

    fn size(&self) -> usize {
        self.as_ref().size()
    }
}
```

Any type which can be interpreted as a reference to `Scannable` is also a scannable! This enables us to perform scans over `Box<dyn i32>`, where the type is known at runtime! Or rather, it would, if `Box<dyn T>` implemented `Clone`, which it can't[^10] because that's what prompted this entire issue. Dang it! I can't catch a breath today!

Okay, let's step back. Why did we need our scannables to be clone in the first place? When we perform exact scans, we store the original value in the region, which we don't own, so we clone it. But what if we *did* own the value? Instead of taking the `Scan` by reference, which holds `T: Scannable`, we could take it by value. If we get rid of all the `Clone` bounds and update `Scan::run` to take `self`, along with updating all the things that take a `Region` to take them by value as well, it should all work out.

But it does not. If we take `Scan` by value, with it not being `Clone`, we simply can't use it to scan over multiple regions. After the first region, we have lost the `Scan`.

Let's take a second step back. We are scanning memory, and we want to compare memory, but we want to treat the memory with different semantics (for example, if we treat it as `f32`, we want to check for rough equality). Instead of storing the *value* itself, we could store its *memory representation*, and when we compare memory representations, we can do so under certain semantics.

First off, let's revert getting rid of all `Clone`. Wherever we stored a `T`, we will now store a `Vec<u8>`. We will still use a type parameter to represent the "implementations of `Scannable`". For this to work, our definitions need to use `T` somewhere, or else the compiler refuses to compile the code with error [E0392]. For this, I will stick a [`PhantomData`][phantom] in the `Exact` variant. It's a bit pointless to include it in all variants, and `Exact` seems the most appropriated:

```rust
pub enum Scan<T: Scannable> {
    Exact(Vec<u8>, PhantomData<T>),
    Unknown,
    ...
}
```

This keeps in line with `Value`:

```rust
pub enum Value<T: Scannable> {
    Exact(Vec<u8>, PhantomData<T>),
    ...
}
```

Our `Scannable` will no longer work on `T` and `&[u8]`. Instead, it will work on two `&[u8]`. We will also need a way to interpret a `T` as `&[u8]`, which we can achieve with a new method, `mem_view`. This method interprets the raw memory representation of `self` as its raw bytes. It also lets us get rid of `size`, because we can simply do `mem_view().len()`. It's still `unsafe` to implement, because it should return the same length every time:

```rust
pub unsafe trait Scannable {
    // Callers must `assert_eq!(left.len(), right.len(), self.mem_view().len())`.
    unsafe fn eq(left: &[u8], right: &[u8]) -> bool;
    unsafe fn cmp(left: &[u8], right: &[u8]) -> Ordering;
    fn mem_view(&self) -> &[u8];
}
```

But now we can't use it in trait object, so the following no longer works:

```rust
unsafe impl<T: AsRef<dyn Scannable> + AsMut<dyn Scannable>> Scannable for T {
    ...
}
```

Ugh! Well, to be fair, we no longer have a "scannable" at this point. It's more like a scan mode that tells us how memory should be compared according to a certain type. Let's split the trait into two: one for the scan mode, and other for "things which are scannable":

```rust
pub trait ScanMode {
    unsafe fn eq(left: &[u8], right: &[u8]) -> bool;
    unsafe fn cmp(left: &[u8], right: &[u8]) -> Ordering;
}

pub unsafe trait Scannable {
    type Mode: ScanMode;

    fn mem_view(&self) -> &[u8];
}
```

Note that we have an associated `type Mode` which contains the corresponding `ScanMode`. If we used a trait bound such as `Scannable: ScanMode`, we'd be back to square one: it would inherit the method definitions that don't use `&self` and thus cannot be used as trait objects.

With these changes, it is possible to implement `Scannable` for any `dyn Scannable`:

```rust
unsafe impl<T: ScanMode + AsRef<dyn Scannable<Mode = Self>>> Scannable for T {
    type Mode = Self;

    fn mem_view(&self) -> &[u8] {
        self.as_ref().mem_view()
    }
}
```

We do have to adjust a few places of the code to account for both `Scannable` and `ScanMode`, but all in all, it's pretty straightforward. Things like `Value` don't need to store the `Scannable` anymore, just a `Vec<u8>`. It also doesn't need the `ScanMode`, because it's not going to be scanning anything on its own. This applies transitively to `Region` which was holding a `Value`.

`Value` *does* need to be updated to store the size of the region we are scanning for, however, because we need that information when running a subsequent scan. For all `Scan` that don't have a explicit thing to scan for (like `Decreased`), the `size` also needs to be stored in them.

Despite all our efforts, we're still unable to return an `Scannable` chosen at runtime.

```rust
fn prompt_user_for_scan() -> Scan<Box<dyn Scannable<Mode = ???>>> {
    todo!()
}
```

As far as I can tell, there's simply no way to specify that type. We want to return a type which is scannable, which has itself (which is also a `ScanMode`) as the corresponding mode. Even if we just tried to return the mode, we simply can't, because it's not object-safe. Is this the end of the road?

## Specifying the scan mode

We need a way to pass an arbitrary scan mode to our `Scan`. This scan mode should go in tandem with `Scannable` types, because it would be unsafe otherwise. We've seen that using a type just doesn't cut it. What else can we do?

Using an enumeration is a no-go, because I want users to be able to extend it further. I also would like to avoid having to update the `enum` and all the matches every time I come up with a different type combination. And it could get pretty complicated if I ever built something dynamically, such as letting the user combine different scans in one pass.

So what if we make `Scannable` return a value that implements the functions we need?

```rust
pub struct ScanMode {
    eq: unsafe fn(left: &[u8], right: &[u8]) -> bool,
    cmp: unsafe fn(left: &[u8], right: &[u8]) -> Ordering,
}
```

It's definitely… non-conventional. But hey, now we're left with the `Scannable` trait, which is object-safe, and does not have any type parameters!

```rust
pub unsafe trait Scannable {
    fn mem_view(&self) -> &[u8];
    fn scan_mode(&self) -> ScanMode;
}
```

It is a bit weird, but defining local functions and using those in the returned value is a nice way to keep things properly scoped:

```rust
macro_rules! impl_scannable_for_int {
    ( $( $ty:ty ),* ) => {
        $(
            unsafe impl Scannable for $ty {
                fn mem_view(&self) -> &[u8] {
                    unsafe { std::slice::from_raw_parts(self as *const _ as *const u8, mem::size_of::<$ty>()) }
                }

                fn scan_mode(&self) -> ScanMode {
                    unsafe fn eq(left: &[u8], right: &[u8]) -> bool {
                        ...
                    }

                    unsafe fn cmp(left: &[u8], right: &[u8]) -> Ordering {
                        ...
                    }

                    ScanMode { eq, cmp }
                }
            }
        )*
    };
}
```

Our `Scan` needs to store the `Scannable` type, and not just the memory, once again. For variants that don't need any value, they can store the `ScanMode` and size instead.

Does this solution work? Yes! It's possible to return a `Box<dyn Scannable>` from a function, and underneath, it may be using any type which is `Scannable`. Is this the best solution? Well, that's hard to say. This is *one* of the possible solutions.

We have been going around in circles for quite some time now, so I'll leave it there. It's a solution, which may not be pretty, but it works. With these changes, the code is capable of completing all of the steps in the Cheat Engine tutorial up until point!

## Finale

If there's one lesson to learn from this post, it's that there is often no single correct solution to a problem. We could have approached the scan types in many, many ways (and we tried quite a few!), but in the end, choosing one option or the other comes down to your (sometimes self-imposed) requirements.

You may [obtain the code for this post][code] over at my GitHub. You can run `git checkout step4` after cloning the repository to get the right version of the code. The code has gone through a lot of iterations, and I'd still like to polish it a bit more, so it might slightly differ from the code presented in this entry.

If you feel adventurous, Cheat Engine has different options for scanning floating point types: "rounded (default)", "rounded (extreme)", and truncated. Optionally, it can scan for "simple values only". You could go ahead and toy around with these!

We didn't touch on types with different lengths, such as strings. You could support UTF-8, UTF-16, or arbitrary byte sequences. This post also didn't cover scanning for multiple things at once, known as "groupscan commands", although from what I can tell, these are just a nice way to scan for arbitrary byte sequences.

We also didn't look into supporting different the same scan with different alignments. All these things may be worth exploring depending on your requirements. You could even get rid of such genericity and go with something way simpler. Supporting `i32`, `f32` and `f64` is enough to complete the Cheat Engine tutorial. But I wanted something more powerful, although my solution currently can't scan for a sequence such as "exact type, unknown, exact matching the unknown". So yeah.

In the [next post](/blog/woce-5), we'll tackle the fifth step of the tutorial: Code finder. Cheat Engine attaches its debugger to the process for this one, and then replaces the instruction that performs the write with a different no-op so that nothing is written anymore. This will be quite the challenge!

### Footnotes

[^1]: [`Copy` and `Drop` are exclusive][copy-drop]. See also [E0184].

[^2]: If you added more scan types that require additional bounds, make sure to add them too. For example, the "decreased by" scan requires the type to `impl Sub`.

[^3]: This is a good time to remind you to read the documentation. It is of special importance when dealing with `unsafe` methods; I recommend reading it a couple times.

[^4]: Even with this option, it would not be a bad idea to make the trait `unsafe`.

[^5]: Not for long. As we will find out later, this approach has its limitations.

[^6]: We can still perform the pointer dereference when we know it's aligned. This would likely be an optimization, although it would definitely complicate the code more.

[^7]: It *would* work if you scanned for unknown values and then checked for decreased values repeatedly. But we can't just leave exact scan broken!

[^8]: Unfortunately, this makes some optimizations harder or even impossible to perform. Providing specialized functions for types where the size is known at compile time could be worth doing. Programming is all tradeoffs.

[^9]: [Rust 1.51][rust151], which was not out at the time of writing, would make it a lot easier to allow scanning for fixed-length sequences of bytes, thanks to const generics.

[^10]: Workarounds do exist, such as [dtolnay's `dyn-clone`][dynclone]. But I would rather not go that route.

[transmute]: https://doc.rust-lang.org/stable/std/mem/fn.transmute.html
[ub]: https://doc.rust-lang.org/stable/reference/behavior-considered-undefined.html
[code]: https://github.com/lonami/memo
[sized-default]: https://doc.rust-lang.org/stable/std/marker/trait.Sized.html
[fromne]: https://doc.rust-lang.org/stable/std/primitive.i32.html#method.from_ne_bytes
[tone]: https://doc.rust-lang.org/stable/std/primitive.i32.html#method.to_ne_bytes
[inv-val]: https://doc.rust-lang.org/nomicon/what-unsafe-does.html
[seal]: https://rust-lang.github.io/api-guidelines/future-proofing.html
[pointer]: https://doc.rust-lang.org/std/primitive.pointer.html
[ptr-read]: https://doc.rust-lang.org/std/ptr/fn.read.html
[ptr-readun]: https://doc.rust-lang.org/std/ptr/fn.read_unaligned.html
[turbofish]: https://www.reddit.com/r/rust/comments/3fimgp/why_double_colon_rather_that_dot/ctozkd0/
[f16]: https://en.wikipedia.org/wiki/Bfloat16_floating-point_format
[objectsafe]: https://doc.rust-lang.org/stable/error-index.html#E0038
[copy-drop]: https://doc.rust-lang.org/stable/std/ops/trait.Drop.html#copy-and-drop-are-exclusive
[E0184]: https://doc.rust-lang.org/stable/error-index.html#E0184
[E0392]: https://doc.rust-lang.org/stable/error-index.html#E0392
[phantom]: https://doc.rust-lang.org/stable/std/marker/struct.PhantomData.html
[rust151]: https://blog.rust-lang.org/2021/02/26/const-generics-mvp-beta.html
[dynclone]: https://crates.io/crates/dyn-clone
