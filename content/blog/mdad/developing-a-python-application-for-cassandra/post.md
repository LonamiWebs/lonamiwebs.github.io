```meta
title: Developing a Python application for Cassandra
published: 2020-03-23T00:00:00+00:00
updated: 2020-04-16T07:52:26+00:00
```

_**Warning**: this post is, in fact, a shameless self-plug to my own library. If you continue reading, you accept that you are okay with this. Otherwise, please close the tab, shut down your computer, and set it on fire.__(Also, that was a joke. Please don’t do that.)_

Let’s do some programming! Today we will be making a tiny CLI application in [Python](http://python.org/) that queries [Telegram’s API](https://core.telegram.org/api) and stores the data in [Cassandra](http://cassandra.apache.org/).

## Our goal

Our goal is to make a Python console application. This application will connect to [Telegram](https://telegram.org/), and ask for your account credentials. Once you have logged in, the application will fetch all of your open conversations and we will store these in Cassandra.

With the data saved in Cassandra, we can now very efficiently query information about your conversations given their identifier offline (no need to query Telegram anymore).

**In short**, we are making an application that performs efficient offline queries to Cassandra to print out information about your Telegram conversations given the ID you want to query.

## Data model

The application itself is really simple, and we only need one table to store all the relevant information we will be needing. This table called `**users**` will contain the following columns:

* `**id**`, of type `int`. This will also be the `primary key` and we’ll use it to query the database later on.
* `**first_name**`, of type `varchar`. This field contains the first name of the stored user.
* `**last_name**`, of type `varchar`. This field contains the last name of the stored user.
* `**username**`, of type `varchar`. This field contains the username of the stored user.
Because Cassandra uses a [wide column storage model](https://cassandra.apache.org/doc/latest/architecture/overview.html), direct access through a key is the most efficient way to query the database. In our case, the key is the primary key of the `users` table, using the `id` column. The index for the primary key is ready to be used as soon as we create the table, so we don’t need to create it on our own.

## Dependencies

Because we will program it in Python, you need Python installed. You can install it using a package manager of your choice or heading over to the [Python downloads section](https://www.python.org/downloads/), but if you’re on Linux, chances are you have it installed already.

Once Python 3.5 or above is installed, get a copy of the Cassandra driver for Python and Telethon through `pip`:

```
pip install cassandra-driver telethon
```

For more details on that, see the [installation guide for `cassandra-driver`](https://docs.datastax.com/en/developer/python-driver/3.22/installation/), or the [installation guide for `telethon`](https://docs.telethon.dev/en/latest/basic/installation.html).

As we did in our [previous post](/blog/mdad/cassandra-operaciones-basicas-y-arquitectura/), we will setup a new keyspace for this application with `cqlsh`. We will also create a table to store the users into. This could all be automated in the Python code, but because it’s a one-time thing, we prefer to use `cqlsh`.

Make sure that Cassandra is running in the background. We can’t make queries to it if it’s not running.

```
$ bin/cqlsh
Connected to Test Cluster at 127.0.0.1:9042.
[cqlsh 5.0.1 | Cassandra 3.11.6 | CQL spec 3.4.4 | Native protocol v4]
Use HELP for help.
cqlsh> create keyspace mdad with replication = {'class': 'SimpleStrategy', 'replication_factor': 3};
cqlsh> use mdad;
cqlsh:mdad> create table users(id int primary key, first_name varchar, last_name varchar, username varchar);
```

Python installed? Check. Python dependencies? Check. Cassandra ready? Check.

## The code

### Getting users

The first step is connecting to [Telegram’s API](https://core.telegram.org/api), for which we’ll use [Telethon](https://telethon.dev/), a wonderful (wink, wink) Python library to interface with it.

As with most APIs, we need to supply [our API key](https://my.telegram.org/) in order to use it (here `API_ID` and `API_HASH`). We will refer to them as constants. At the end, you may download the entire code and use my own key for this example. But please don’t use those values for your other applications!

It’s pretty simple: we create a client, and for every dialog (that is, open conversation) we have, do some checks:

* If it’s an user, we just store that in a dictionary mapping `ID → User`.
* Else if it’s a group, we iterate over the participants and store those users instead.

```
async def load_users():
    from telethon import TelegramClient

    users = {}

    async with TelegramClient(SESSION, API_ID, API_HASH) as client:
        async for dialog in client.iter_dialogs():
            if dialog.is_user:
                user = dialog.entity
                users[user.id] = user
                print('found user:', user.id, file=sys.stderr)

            elif dialog.is_group:
                async for user in client.iter_participants(dialog):
                    users[user.id] = user
                    print('found member:', user.id, file=sys.stderr)

    return list(users.values())
```

With this we have a mapping ID to user, so we know we won’t have duplicates. We simply return the list of user values, because that’s all we care about.

### Saving users

Inserting users into Cassandra is pretty straightforward. We take the list of `User` objects as input, and prepare a new `INSERT` statement that we can reuse (because we will be using it in a loop, this is the best way to do it).

For each user, execute the statement with the user data as input parameters. Simple as that.

```
def save_users(session, users):
    insert_stmt = session.prepare(
        'INSERT INTO users (id, first_name, last_name, username) ' 
        'VALUES (?, ?, ?, ?)')

    for user in users:
        row = (user.id, user.first_name, user.last_name, user.username)
        session.execute(insert_stmt, row)
```

### Fetching users

Given a list of users, yield all of them from the database. Similar to before, we prepare a `SELECT` statement and just execute it repeatedly over the input user IDs.

```
def fetch_users(session, users):
    select_stmt = session.prepare('SELECT * FROM users WHERE id = ?')

    for user_id in users:
        yield session.execute(select_stmt, (user_id,)).one()
```

### Parsing arguments

We’ll be making a little CLI application, so we need to parse console arguments. It won’t be anything fancy, though. For that we’ll be using [Python’s `argparse` module](https://docs.python.org/3/library/argparse.html):

```
def parse_args():
    import argparse

    parser = argparse.ArgumentParser(
        description='Dump and query Telegram users')

    parser.add_argument('users', type=int, nargs='*',
        help='one or more user IDs to query for')

    parser.add_argument('--load-users', action='store_true',
        help='load users from Telegram (do this first run)')

    return parser.parse_args()
```

### All together

Last, the entry point. We import a Cassandra Cluster, and connect to some default keyspace (we called it `mdad` earlier).

If the user wants to load the users into the database, we’ll do just that first.

Then, for each user we fetch from the database, we print it. Last names and usernames are optional, so don’t print those if they’re missing (`None`).

```
async def main(args):
    from cassandra.cluster import Cluster

    cluster = Cluster(CLUSTER_NODES)
    session = cluster.connect(KEYSPACE)

    if args.load_users:
        users = await load_users()
        save_users(session, users)

    for user in fetch_users(session, args.users):
        print('User', user.id, ':')
        print('  First name:', user.first_name)
        if user.last_name:
            print('  Last name:', user.last_name)
        if user.username:
            print('  Username:', user.username)

        print()

if __name__ == '__main__':
    asyncio.run(main(parse_args()))
```

Because Telethon is an `[asyncio](https://docs.python.org/3/library/asyncio.html)` library, we define it as `async def main(...)` and run it with `asyncio.run(main(...))`.

Here’s what it looks like in action:

```
$ python data.py --help
usage: data.py [-h] [--load-users] [users [users ...]]

Dump and query Telegram users

positional arguments:
  users         one or more user IDs to query for

optional arguments:
  -h, --help    show this help message and exit
  --load-users  load users from Telegram (do this first run)

$ python data.py --load-users
found user: 487158
found member: 59794114
found member: 487158
found member: 191045991
(...a lot more output)

$ python data.py 487158 59794114
User 487158 :
  First name: Rick
  Last name: Pickle

User 59794114 :
  Firt name: Peter
  Username: pete
```

Telegram’s data now persists in Cassandra, and we can efficiently query it whenever we need to! I would’ve shown a video presenting its usage, but I’m afraid that would leak some of the data I want to keep private :-).

Feel free to download the code and try it yourself:

*download removed*

## References

* [DataStax Python Driver for Apache Cassandra – Getting Started](https://docs.datastax.com/en/developer/python-driver/3.22/getting_started/)
* [Telethon’s Documentation](https://docs.telethon.dev/en/latest/)
