+++
title = "An Introduction to Asyncio"
date = 2018-06-13
updated = 2023-11-22
[taxonomies]
category = ["sw"]
tags = ["python", "asyncio"]
+++

Index
-----

* [Background](#background)
* [Input / Output](#input-output)
* [Diving In](#diving-in)
* [A Toy Example](#a-toy-example)
* [A Real Example](#a-real-example)
* [Extra Material](#extra-material)


Background
----------

After seeing some friends struggle with `asyncio` I decided that it could be a good idea to write a blog post using my own words to explain how I understand the world of asynchronous IO. I will focus on Python's `asyncio` module but this post should apply to any other language easily.

So what is `asyncio` and what makes it good? Why don't we just use the old and known threads to run several parts of the code concurrently, at the same time?

The first reason is that `asyncio` makes your code easier to reason about, as opposed to using threads, because the amount of ways in which your code can run grows exponentially. Let's see that with an example. Imagine you have this code:

```python
def method():
	line 1
	line 2
	line 3
	line 4
	line 5
```

And you start two threads to run the method at the same time. What is the order in which the lines of code get executed? The answer is that you can't know! The first thread can run the entire method before the second thread even starts. Or it could be the first thread that runs after the second thread. Perhaps both run the "line 1", and then the line 2. Maybe the first thread runs lines 1 and 2, and then the second thread only runs the line 1 before the first thread finishes.

As you can see, any combination of the order in which the lines run is possible. If the lines modify some global shared state, that will get messy quickly.

Second, in Python, threads *won't* make your code faster most of the time. It will only increase the concurrency of your program (which is okay if it makes many blocking calls), allowing you to run several things at the same time.

If you have a lot of CPU work to do though, threads aren't a real advantage. Indeed, your code will probably run slower under the most common Python implementation, CPython, which makes use of a Global Interpreter Lock (GIL) that only lets a thread run at once. The operations won't run in parallel!

Input / Output
--------------

Before we go any further, let's first stop to talk about input and output, commonly known as "IO". There are two main ways to perform IO operations, such as reading or writing from a file or a network socket.

The first one is known as "blocking IO". What this means is that, when you try performing IO, the current application thread is going to *block* until the Operative System can tell you it's done. Normally, this is not a problem, since disks are pretty fast anyway, but it can soon become a performance bottleneck. And network IO will be much slower than disk IO!

```python
import socket

# Setup a network socket and a very simple HTTP request.
# By default, sockets are open in blocking mode.
sock = socket.socket()
request = b'''HEAD / HTTP/1.0\r
Host: example.com\r
\r
'''

# "connect" will block until a successful TCP connection
# is made to the host "example.com" on port 80.
sock.connect(('example.com', 80))

# "sendall" will repeatedly call "send" until all the data in "request" is
# sent to the host we just connected, which blocks until the data is sent.
sock.sendall(request)

# "recv" will try to receive up to 1024 bytes from the host, and block until
# there is any data to receive (or empty if the host closes the connection).
response = sock.recv(1024)

# After all those blocking calls, we got out data! These are the headers from
# making a HTTP request to example.com.
print(response.decode())
```

Blocking IO offers timeouts, so that you can get control back in your code if the operation doesn't finish. Imagine that the remote host doesn't want to reply, your code would be stuck for as long as the connection remains alive!

But wait, what if we make the timeout small? Very, very small? If we do that, we will never block waiting for an answer. That's how asynchronous IO works, and it's the opposite of blocking IO (you can also call it non-blocking IO if you want to).

How does non-blocking IO work if the IO device needs a while to answer with the data? In that case, the operative system responds with "not ready", and your application gets control back so it can do other stuff while the IO device completes your request. It works a bit like this:

```
<app> Hey, I would like to read 16 bytes from this file
<OS> Okay, but the disk hasn't sent me the data yet
<app> Alright, I will do something else then
(a lot of computer time passes)
<app> Do you have my 16 bytes now?
<OS> Yes, here they are! "Hello, world !!\n"
```

In reality, you can tell the OS to notify you when the data is ready, as opposed to polling (constantly asking the OS whether the data is ready yet or not), which is more efficient.

But either way, that's the difference between blocking and non-blocking IO, and what matters is that your application gets to run more without ever needing to wait for data to arrive, because the data will be there immediately when you ask, and if it's not yet, your app can do more things meanwhile.


Diving In
---------

Now we've seen what blocking and non-blocking IO is, and how threads make your code harder to reason about, but they give concurrency (yet not more speed). Is there any other way to achieve this concurrency that doesn't involve threads? Yes! The answer is `asyncio`.

So how does `asyncio` help? First we need to understand a very crucial concept before we can dive any deeper, and I'm talking about the *event loop*. What is it and why do we need it?

You can think of the event loop as a *loop* that will be responsible for calling your `async` functions:

![The Event Loop](eventloop.svg)

That's silly you may think. Now not only we run our code but we also have to run some "event loop". It doesn't sound beneficial at all. What are these events? Well, they are the IO events we talked about before!

`asyncio`'s event loop is responsible for handling those IO events, such as file is ready, data arrived, flushing is done, and so on. As we saw before, we can make these events non-blocking by setting their timeout to 0.

Let's say you want to read from 10 files at the same time. You will ask the OS to read data from 10 files, and at first none of the reads will be ready. But the event loop will be constantly asking the OS to know which are done, and when they are done, you will get your data.

This has some nice advantages. It means that, instead of waiting for a network request to send you a response or some file, instead of blocking there, the event loop can decide to run other code meanwhile. Whenever the contents are ready, they can be read, and your code can continue. Waiting for the contents to be received is done with the `await` keyword, and it tells the loop that it can run other code meanwhile:

![Step 1, await keyword](awaitkwd1.svg)

![Step 2, await keyword](awaitkwd2.svg)

Start reading the code of the event loop and follow the arrows. You can see that, in the beginning, there are no events yet, so the loop calls one of your functions. The code runs until it has to `await` for some IO operation to complete, such as sending a request over the network. The method is "paused" until an event occurs (for example, an "event" occurs when the request has been sent completely).

While the first method is busy, the event loop can enter the second method, and run its code until the first `await`. But it can happen that the event of the second query occurs before the request on the first method, so the event loop can re-enter the second method because it has already sent the query, but the first method isn't done sending the request yet.

Then, the second method `await`'s for an answer, and an event occurs telling the event loop that the request from the first method was sent. The code can be resumed again, until it has to `await` for a response, and so on. Here's an explanation with pseudo-code for this process if you prefer:

```python
async def method(request):
    prepare request
    await send request

    await receive request

    process request
    return result

run concurrently (
	method with request 1,
	method with request 2,
)
```

This is what the event loop will do on the above pseudo-code:

```
no events pending, can advance

enter method with request 1
	prepare request
	await sending request
pause method with request 1

no events ready, can advance

enter method with request 2
	prepare request
	await sending request
pause method with request 2

both requests are paused, cannot advance
wait for events
event for request 2 arrives (sending request completed)

enter method with request 2
	await receiving response
pause method with request 2

event for request 1 arrives (sending request completed)

enter method with request 1
	await receiving response
pause method with request 1

...and so on
```

You may be wondering "okay, but threads work for me, so why should I change?". There are some important things to note here. The first is that we only need one thread to be running! The event loop decides when and which methods should run. This results in less pressure for the operating system. The second is that we know when it may run other methods. Those are the `await` keywords! Whenever there is one of those, we know that the loop is able to run other things until the resource (again, like network) becomes ready (when a event occurs telling us it's ready to be used without blocking or it has completed).

So far, we already have two advantages. We are only using a single thread so the cost for switching between methods is low, and we can easily reason about where our program may interleave operations.

Another advantage is that, with the event loop, you can easily schedule when a piece of code should run, such as using the method [`loop.call_at`](https://docs.python.org/3/library/asyncio-eventloop.html#asyncio.loop.call_at), without the need for spawning another thread at all.

To tell the `asyncio` to run the two methods shown above, we can use [`asyncio.ensure_future`](https://docs.python.org/3/library/asyncio-future.html#asyncio.ensure_future), which is a way of saying "I want the future of my method to be ensured". That is, you want to run your method in the future, whenever the loop is free to do so. This method returns a `Future` object, so if your method returns a value, you can `await` this future to retrieve its result.

What is a `Future`? This object represents the value of something that will be there in the future, but might not be there yet. Just like you can `await` your own `async def` functions, you can `await` these `Future`'s.

The `async def` functions are also called "coroutines", and Python does some magic behind the scenes to turn them into such. The coroutines can be `await`'ed, and this is what you normally do.


A Toy Example
-------------

That's all about `asyncio`! Let's wrap up with some example code. We will create a server that replies with the text a client sends, but reversed. First, we will show what you could write with normal synchronous code, and then we will port it.

Here is the **synchronous version**:

```python
# server.py
import socket


def server_method():
	# create a new server socket to listen for connections
	server = socket.socket()

	# bind to localhost:6789 for new connections
	server.bind(('localhost', 6789))

	# we will listen for one client at most
	server.listen(1)

	# *block* waiting for a new client
	client, _ = server.accept()

	# *block* waiting for some data
	data = client.recv(1024)

	# reverse the data
	data = data[::-1]

	# *block* sending the data
	client.sendall(data)

	# close client and server
	server.close()
	client.close()


if __name__ == '__main__':
	# block running the server
	server_method()
```

```python
# client.py
import socket


def client_method():
	message = b'Hello Server!\n'
	client = socket.socket()

	# *block* trying to stabilish a connection
	client.connect(('localhost', 6789))

	# *block* trying to send the message
	print('Sending', message)
	client.sendall(message)

	# *block* until we receive a response
	response = client.recv(1024)
	print('Server replied', response)

	client.close()


if __name__ == '__main__':
	client_method()
```

From what we've seen, this code will block on all the lines with a comment above them saying that they will block. This means that for running more than one client or server, or both in the same file, you will need threads. But we can do better, we can rewrite it into `asyncio`!

The first step is to mark all your `def`initions that may block with `async`. This marks them as coroutines, which can be `await`ed on.

Second, since we're using low-level sockets, we need to make use of the methods that `asyncio` provides directly. If this was a third-party library, this would be just like using their `async def`initions.

Here is the **asynchronous version**:

```python
# server.py
import asyncio
import socket

# get the default "event loop" that we will run
loop = asyncio.get_event_loop()


# notice our new "async" before the definition
async def server_method():
	server = socket.socket()
	server.bind(('localhost', 6789))
	server.listen(1)

	# await for a new client
	# the event loop can run other code while we wait here!
	client, _ = await loop.sock_accept(server)

	# await for some data
	data = await loop.sock_recv(client, 1024)
	data = data[::-1]

	# await for sending the data
	await loop.sock_sendall(client, data)

	server.close()
	client.close()


if __name__ == '__main__':
	# run the loop until "server method" is complete
	loop.run_until_complete(server_method())
```

```python
# client.py
import asyncio
import socket

loop = asyncio.get_event_loop()


async def client_method():
	message = b'Hello Server!\n'
	client = socket.socket()

	# await to stabilish a connection
	await loop.sock_connect(client, ('localhost', 6789))

	# await to send the message
	print('Sending', message)
	await loop.sock_sendall(client, message)

	# await to receive a response
	response = await loop.sock_recv(client, 1024)
	print('Server replied', response)

	client.close()


if __name__ == '__main__':
	loop.run_until_complete(client_method())
```

That's it! You can place these two files separately and run, first the server, then the client. You should see output in the client.

The big difference here is that you can easily modify the code to run more than one server or clients at the same time. Whenever you `await` the event loop will run other of your code. It seems to "block" on the `await` parts, but remember it's actually jumping to run more code, and the event loop will get back to you whenever it can.

In short, you need an `async def` to `await` things, and you run them with the event loop instead of calling them directly. So this…

```python
def main():
	...  # some code


if __name__ == '__main__':
	main()
```

…becomes this:

```python
import asyncio


async def main():
	...  # some code


if __name__ == '__main__':
	asyncio.get_event_loop().run_until_complete(main)
```

This is pretty much how most of your `async` scripts will start, running the main method until its completion.


A Real Example
--------------

Let's have some fun with a real library. We'll be using [Telethon](https://github.com/LonamiWebs/Telethon) to broadcast a message to our three best friends, all at the same time, thanks to the magic of `asyncio`. We'll dive right into the code, and then I'll explain our new friend `asyncio.wait(...)`:

```python
# broadcast.py
import asyncio
import sys

from telethon import TelegramClient

# (you need your own values here, check Telethon's documentation)
api_id = 123
api_hash = '123abc'
friends = [
	'@friend1__username',
	'@friend2__username',
	'@bestie__username'
]

# we will have to await things, so we need an async def
async def main(message):
	# start is a coroutine, so we need to await it to run it
	client = await TelegramClient('me', api_id, api_hash).start()

	# wait for all three client.send_message to complete
	await asyncio.wait([
		client.send_message(friend, message)
		for friend in friends
	])

	# and close our client
	await client.disconnect()


if __name__ == '__main__':
	if len(sys.argv) != 2:
		print('You must pass the message to broadcast!')
		quit()

	message = sys.argv[1]
	asyncio.get_event_loop().run_until_complete(main(message))
```

Wait… how did that send a message to all three of
my friends? The magic is done here:

```python
[
	client.send_message(friend, message)
	for friend in friends
]
```

This list comprehension creates another list with three
coroutines, the three `client.send_message(...)`.
Then we just pass that list to `asyncio.wait`:

```python
await asyncio.wait([...])
```

This method, by default, waits for the list of coroutines to run until they've all finished. You can read more on the Python [documentation](https://docs.python.org/3/library/asyncio-task.html#asyncio.wait). Truly a good function to know about!

Now whenever you have some important news for your friends, you can simply `python3 broadcast.py 'I bought a car!'` to tell all your friends about your new car! All you need to remember is that you need to `await` on coroutines, and you will be good. `asyncio` will warn you when you forget to do so.


Extra Material
--------------

If you want to understand how `asyncio` works under the hood, I recommend you to watch this hour-long talk [Get to grips with asyncio in Python 3](https://youtu.be/M-UcUs7IMIM) by Robert Smallshire. In the video, they will explain the differences between concurrency and parallelism, along with others concepts, and how to implement your own `asyncio` "scheduler" from scratch.
