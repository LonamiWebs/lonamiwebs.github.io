```meta
created: 2019-06-19
```

Python ctypes and Windows
=========================

[Python](https://www.python.org/)'s [`ctypes`](https://docs.python.org/3/library/ctypes.html) is quite a nice library to easily load and invoke C methods available in already-compiled [`.dll` files](https://en.wikipedia.org/wiki/Dynamic-link_library) without any additional dependencies. And I *love* depending on as little as possible.

In this blog post, we will walk through my endeavors to use `ctypes` with the [Windows API](https://docs.microsoft.com/en-us/windows/desktop/api/), and do some cool stuff with it.

We will assume some knowledge of C/++ and Python, since we will need to read and write a bit of both. Please note that this post is only an introduction to `ctypes`, and if you need more information you should consult the [Python's documentation for `ctypes`](https://docs.python.org/3/library/ctypes.html).

While the post focuses on Windows' API, the code here probably applies to unix-based systems with little modifications.

Basics
------

First of all, let's learn how to load a library. Let's say we want to load `User32.dll`:

```python
import ctypes

ctypes.windll.user32
```

Yes, it's that simple. When you access an attribute of `windll`, said library will load. Since Windows is case-insensitive, we will use lowercase consistently.

Calling a function is just as simple. Let's say you want to call [`SetCursorPos`](https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-setcursorpos), which is defined as follows:
```c
BOOL SetCursorPos(
    int X,
    int Y
);
```

Okay, it returns a `bool` and takes two inputs, `x` and `y`. So we can call it like so:

```python
ctypes.windll.user32.SetCursorPos(100, 100)
```

Try it! Your cursor will move!

Funky Stuff
-----------

We can go a bit more crazy and make it form a spiral:

```python
import math
import time

for i in range(200):
    x = int(500 + math.cos(i / 5) * i)
    y = int(500 + math.sin(i / 5) * i)
    ctypes.windll.user32.SetCursorPos(x, y)
    time.sleep(0.05)
```

Ah, it's always so pleasant to do random stuff when programming. Sure makes it more fun.

Complex Structures
------------------

`SetCursorPos` was really simple. It took two parameters and they both were integers. Let's go with something harder. Let's go with [`SendInput`](https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-sendinput)! Emulating input will be a fun exercise:

```c
UINT SendInput(
    UINT    cInputs,
    LPINPUT pInputs,
    int     cbSize
);
```

Okay, `LPINPUT`, what are you? Microsoft likes to prefix types with what they are. In this case, `LP` stands for "Long Pointer" (I guess?), so `LPINPUT` is just a Long Pointer to [`INPUT`](https://docs.microsoft.com/en-us/windows/desktop/api/winuser/ns-winuser-taginput):

```c
typedef struct tagINPUT {
    DWORD type;
    union {
        MOUSEINPUT    mi;
        KEYBDINPUT    ki;
        HARDWAREINPUT hi;
    } DUMMYUNIONNAME;
} INPUT, *PINPUT, *LPINPUT;
```

Alright, that's new. We have a `struct` and `union`, two different concepts. We can define both with `ctypes`:

```python
INPUT_MOUSE = 0
INPUT_KEYBOARD = 1
INPUT_HARDWARE = 2

class INPUT(ctypes.Structure):
    _fields_ = [
        ('type', ctypes.c_long),
        ...
    ]
```

Structures are classes that subclass `ctypes.Structure`, and you define their fields in the `_fields_` class-level variable, which is a list of tuples `(field name, field type)`.

The C structure had a `DWORD type`. `DWORD` is a `c_long`, and `type` is a name like any other, which is why we did `('type', ctypes.c_long)`.

But what about the union? It's anonymous, and we can't make anonymous unions (*citation needed*) with `ctypes`. We will give it a concrete name and a type.

Before defining the union, we need to define its inner structures, [`MOUSEINPUT`](https://docs.microsoft.com/en-us/windows/desktop/api/winuser/ns-winuser-tagmouseinput), [`KEYBDINPUT`](https://docs.microsoft.com/en-us/windows/desktop/api/winuser/ns-winuser-tagkeybdinput) and [`HARDWAREINPUT`](https://docs.microsoft.com/en-us/windows/desktop/api/winuser/ns-winuser-taghardwareinput). We won't be using them all, but since they count towards the final struct size (C will choose the largest structure as the final size), we need them, or Windows' API will get confused and refuse to work (personal experience):

```python
class MOUSEINPUT(ctypes.Structure):
    _fields_ = [
        ('dx', ctypes.c_long),
        ('dy', ctypes.c_long),
        ('mouseData', ctypes.c_long),
        ('dwFlags', ctypes.c_long),
        ('time', ctypes.c_long),
        ('dwExtraInfo', ctypes.POINTER(ctypes.c_ulong))
    ]


class KEYBDINPUT(ctypes.Structure):
    _fields_ = [
        ('wVk', ctypes.c_short),
        ('wScan', ctypes.c_short),
        ('dwFlags', ctypes.c_long),
        ('time', ctypes.c_long),
        ('dwExtraInfo', ctypes.POINTER(ctypes.c_ulong))
    ]


class HARDWAREINPUT(ctypes.Structure):
    _fields_ = [
        ('uMsg', ctypes.c_long),
        ('wParamL', ctypes.c_short),
        ('wParamH', ctypes.c_short)
    ]


class INPUTUNION(ctypes.Union):
    _fields_ = [
        ('mi', MOUSEINPUT),
        ('ki', KEYBDINPUT),
        ('hi', HARDWAREINPUT)
    ]


class INPUT(ctypes.Structure):
    _fields_ = [
        ('type', ctypes.c_long),
        ('value', INPUTUNION)
    ]
```

Some things to note:
- Pointers are defined as `ctypes.POINTER(inner type)`.
- The field names can be anything you want. You can make them more "pythonic" if you want (such as changing `dwExtraInfo` for just `extra_info`), but I chose to stick with the original naming.
- The union is very similar, but it uses `ctypes.Union` instead of `ctypes.Structure`.
- We gave a name to the anonymous union, `INPUTUNION`, and used it inside `INPUT` with also a made-up name, `('value', INPUTUNION)`.

Now that we have all the types we need defined, we can use them:

```python
KEYEVENTF_KEYUP = 0x0002

def press(vk, down):
    inputs = INPUT(type=INPUT_KEYBOARD, value=INPUTUNION(ki=KEYBDINPUT(
        wVk=vk,
        wScan=0,
        dwFlags=0 if down else KEYEVENTF_KEYUP,
        time=0,
        dwExtraInfo=None
    )))
    ctypes.windll.user32.SendInput(1, ctypes.byref(inputs), ctypes.sizeof(inputs))


for char in 'HELLO':
    press(ord(char), down=True)
    press(ord(char), down=False)
```

Run it! It will press and release the keys `hello` to type the word `"hello"`!

`vk` stands for "virtual key". Letters correspond with their upper-case ASCII value, which is what we did above. You can find all the available keys in the page with all the [Virtual Key Codes](https://docs.microsoft.com/en-us/windows/desktop/inputdev/virtual-key-codes).

Dynamic Inputs and Pointers
---------------------------

What happens if a method wants something by reference? That is, a pointer to your thing? For example, [`GetCursorPos`](https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-getcursorpos):

```c
typedef struct tagPOINT {
    LONG x;
    LONG y;
} POINT, *PPOINT, *NPPOINT, *LPPOINT;

BOOL GetCursorPos(
    LPPOINT lpPoint
);
```

It wants a Long Pointer to [`POINT`](https://docs.microsoft.com/en-us/windows/desktop/api/windef/ns-windef-point). We can do just that with `ctypes.byref`:

```python
class POINT(ctypes.Structure):
    _fields_ = [
        ('x', ctypes.c_long),
        ('y', ctypes.c_long)
    ]


def get_mouse():
    point = POINT()
    ctypes.windll.user32.GetCursorPos(ctypes.byref(point))
    #                  pass our point by ref ^^^^^
    # this lets GetCursorPos fill its x and y fields

    return point.x, point.y


while True:
    print(get_mouse())
    time.sleep(0.05)
```

Now you can track the mouse position! Make sure to `Ctrl+C` the program when you're tired of it.

What happens if a method wants a dynamically-sized input?

```python
buffer = ctypes.create_string_buffer(size)
```

In that case, you can create an in-memory `buffer` of `size` with `ctypes.create_string_buffer`. It will return a character array of that size, which you can pass as a pointer directly (without `ctypes.byref`).

To access the buffer's contents, you can use either `.raw` or `.value`:

```python
entire_buffer_as_bytes = buffer.raw
up_until_null = buffer.value
```

When the method fills in the data, you can `cast` your buffer back into a pointer of a concrete type:

```python
result_ptr = ctypes.cast(buffer, ctypes.POINTER(ctypes.c_long))
```

And you can de-reference pointers with `.contents`:

```python
first_result = result_ptr.contents
```

Arrays
------

Arrays are defined as `type * size`. Your linter may not like that, and if you don't know the size beforehand, consider creating a 0-sized array. For example:

```python
# 10 longs
ten_longs = (ctypes.c_long * 10)()
for i in range(10):
    ten_longs[i] = 2 ** i

# Unknown size of longs, e.g. inside some Structure
longs = (ctypes.c_long * 0)

# Now you know how many longs it actually was
known_longs = ctypes.cast(
    ctypes.byref(longs),
    ctypes.POINTER(ctypes.c_long * size)
).contents
```

If there's a better way to initialize arrays, please let me know.

wintypes
--------

Under Windows, the `ctypes` module has a `wintypes` submodule. This one contains definitions like `HWND` which may be useful and can be imported as:

```python
from ctypes.wintypes import HWND, LPCWSTR, UINT
```

Callbacks
---------

Some functions (I'm looking at you, <a href="https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumwindows"><code>EnumWindows</code></a>) ask us to pass a callback. In this case, it wants a <a href="https://docs.microsoft.com/en-us/previous-versions/windows/desktop/legacy/ms633498(v=vs.85)"><code>EnumWindowsProc</code></a>:

```c
BOOL EnumWindows(
    WNDENUMPROC lpEnumFunc,
    LPARAM      lParam
);

BOOL CALLBACK EnumWindowsProc(
    _In_ HWND   hwnd,
    _In_ LPARAM lParam
);
```

The naive approach won't work:

```python
def callback(hwnd, lParam):
    print(hwnd)
    return True

ctypes.windll.user32.EnumWindows(callback, 0)
# ctypes.ArgumentError: argument 1: <class 'TypeError'>: Don't know how to convert parameter 1
# Aww.
```

Instead, you must wrap your function as a C definition like so:

```python
from ctypes.wintypes import BOOL, HWND, LPARAM

EnumWindowsProc = ctypes.WINFUNCTYPE(BOOL, HWND, LPARAM)

def callback(hwnd, lParam):
    print(hwnd)
    return True

# Wrap the function in the C definition
callback = EnumWindowsProc(callback)

ctypes.windll.user32.EnumWindows(callback, 0)
# Yay, it works.
```

You may have noticed this is what decorators do, wrap the function. So…

```python
from ctypes.wintypes import BOOL, HWND, LPARAM

@ctypes.WINFUNCTYPE(BOOL, HWND, LPARAM)
def callback(hwnd, lParam):
    print(hwnd)
    return True

ctypes.windll.user32.EnumWindows(callback, 0)
```

…will also work. And it is a *lot* fancier.

Closing Words
-------------

With the knowledge above and some experimentation, you should be able to call and do (almost) anything you want. That was pretty much all I needed on my project anyway :)

We have been letting Python convert Python values into C values, but you can do so explicitly too. For example, you can use `ctypes.c_short(17)` to make sure to pass that `17` as a `short`. And if you have a `c_short`, you can convert or cast it to its Python `.value` as `some_short.value`. The same applies for integers, longs, floats, doubles… pretty much anything, char pointers (strings) included.

If you can't find something in their online documentation, you can always [`rg`](https://github.com/BurntSushi/ripgrep) for it in the `C:\Program Files (x86)\Windows Kits\10\Include\*` directory.

Note that the `ctypes.Structure`'s that you define can have more methods of your own. For example, you can write them a `__str__` to easily view its fields, or define a `@property` to re-interpret some data in a meaningful way.

For enumerations, you can pass just the right integer number, make a constant for it, or if you prefer, use a [`enum.IntEnum`](https://docs.python.org/3/library/enum.html#enum.IntEnum). For example, [`DismLogLevel`](https://docs.microsoft.com/en-us/windows-hardware/manufacture/desktop/dism/dismloglevel-enumeration) would be:

```python
class DismLogLevel(enum.IntEnum):
    DismLogErrors = 0
    DismLogErrorsWarnings = 1
    DismLogErrorsWarningsInfo = 2
```

And you *should* be able to pass `DismLogLevel.DismLogErrors` as the parameter now.

If you see a function definition like `Function(void)`, that's C's way of saying it takes no parameters, so just call it as `Function()`.

Make sure to pass all parameters, even if they seem optional they probably still want a `NULL` at least, and of course, read the documentation well. Some methods have certain pre-conditions.

Have fun hacking!
