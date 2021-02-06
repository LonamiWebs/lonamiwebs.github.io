+++
title = "MongoDB: Basic Operations and Architecture"
date = 2020-03-05T04:00:08+00:00
updated = 2020-04-08T17:36:25+00:00
+++

This is the second post in the MongoDB series, where we will take a look at the [CRUD operations](https://stackify.com/what-are-crud-operations/) they support, the data model and architecture used.

Other posts in this series:

* [MongoDB: an Introduction](/blog/ribw/mongodb-an-introduction/)
* [MongoDB: Basic Operations and Architecture](/blog/ribw/mongodb-basic-operations-and-architecture/) (this post)
* [Developing a Python application for MongoDB](/blog/ribw/developing-a-python-application-for-mongodb/)

This post is co-authored wih Classmate, and in it we will take an explorative approach using the `mongo` command line shell to execute commands against the database. It even has TAB auto-completion, which is awesome!

----------

Before creating any documents, we first need to create somewhere for the documents to be in. And before we create anything, the database has to be running, so let’s do that first. If we don’t have a service installed, we can run the `mongod` command ourselves in some local folder to make things easier:

```
$ mkdir -p mongo-database
$ mongod --dbpath mongo-database
```

Just like that, we will have Mongo running. Now, let’s connect to it using the `mongo` command in another terminal (don’t close the terminal where the server is running, we need it!). By default, it connects to localhost, which is just what we need.

```
$ mongo
```

## Create

### Create a database

Let’s list the databases:

```
> show databases
admin   0.000GB
config  0.000GB
local   0.000GB
```

Oh, how interesting! There’s already some databases, even though we just created the directory where Mongo will store everything. However, they seem empty, which make sense.

Creating a new database is done by `use`-ing a name that doesn’t exist. Let’s call our new database «helloworld».

```
> use helloworld
switched to db helloworld
```

Good! Now the «local variable» called `db` points to our `helloworld` database.

```
> db
helloworld
```

What happens if we print the databases again? Surely our new database will show up now…

```
> show databases
admin   0.000GB
config  0.000GB
local   0.000GB
```

…maybe not! It seems Mongo won’t create the database until we create some collections and documents in it. Databases contain collections, and inside collections (which you can think of as tables) we can insert new documents (which you can think of as rows). Like in many programming languages, the dot operator is used to access these «members».

### Create a document

Let’s add a new greeting into the `greetings` collection:

```
> db.greetings.insert({message: "¡Bienvenido!", lang: "es"})
WriteResult({ "nInserted" : 1 })

> show collections
greetings

> show databases
admin       0.000GB
config      0.000GB
helloworld  0.000GB
local       0.000GB
```

That looks promising! We can also see our new `helloworld` database also shows up. The Mongo shell actually works on JavaScript-like code, which is why we can use a variant of JSON (BSON) to insert documents (note the lack of quotes around the keys, convenient!).

The [`insert`](https://docs.mongodb.com/manual/reference/method/db.collection.insert/index.html) method actually supports a list of documents, and by default Mongo will assign a unique identifier to each. If we don’t want that though, all we have to do is add the `_id` key to our documents.

```
> db.greetings.insert([
... {message: "Welcome!", lang: "en"},
... {message: "Bonjour!", lang: "fr"},
... ])
BulkWriteResult({
    "writeErrors" : [ ],
    "writeConcernErrors" : [ ],
    "nInserted" : 2,
    "nUpserted" : 0,
    "nMatched" : 0,
    "nModified" : 0,
    "nRemoved" : 0,
    "upserted" : [ ]
})
```

### Create a collection

In this example, we created the collection `greetings` implicitly, but behind the scenes Mongo made a call to [`createCollection`](https://docs.mongodb.com/manual/reference/method/db.createCollection/). Let’s do just that:

```
> db.createCollection("goodbyes")
{ "ok" : 1 }

> show collections
goodbyes
greetings
```

The method actually has a default parameter to configure other options, like the maximum size of the collection or maximum amount of documents in it, validation-related options, and so on. These are all described in more details in the documentation.

## Read

To read the contents of a document, we have to [`find`](https://docs.mongodb.com/manual/reference/method/db.collection.find/index.html) it.

```
> db.greetings.find()
{ "_id" : ObjectId("5e74829a0659f802b15f18dd"), "message" : "¡Bienvenido!", "lang" : "es" }
{ "_id" : ObjectId("5e7487b90659f802b15f18de"), "message" : "Welcome!", "lang" : "en" }
{ "_id" : ObjectId("5e7487b90659f802b15f18df"), "message" : "Bonjour!", "lang" : "fr" }
```

That’s a bit unreadable for my taste, can we make it more [`pretty`](https://docs.mongodb.com/manual/reference/method/cursor.pretty/index.html)?

```
> db.greetings.find().pretty()
{
    "_id" : ObjectId("5e74829a0659f802b15f18dd"),
    "message" : "¡Bienvenido!",
    "lang" : "es"
}
{
    "_id" : ObjectId("5e7487b90659f802b15f18de"),
    "message" : "Welcome!",
    "lang" : "en"
}
{
    "_id" : ObjectId("5e7487b90659f802b15f18df"),
    "message" : "Bonjour!",
    "lang" : "fr"
}
```

Gorgeous! We can clearly see Mongo created an identifier for us automatically. The queries are also JSON, and support a bunch of operators (prefixed by `$`), known as [Query Selectors](https://docs.mongodb.com/manual/reference/operator/query/). Here’s a few:

<table>
 <thead>
  <tr>
   <th>
    Operation
   </th>
   <th>
    Syntax
   </th>
   <th>
    RDBMS equivalent
   </th>
  </tr>
 </thead>
 <tbody>
  <tr>
   <td>
    Equals
   </td>
   <td>
    <code>
     {key: {$eq: value}}
    </code>
    <br/>
    Shorthand:
    <code>
     {key: value}
    </code>
   </td>
   <td>
    <code>
     where key = value
    </code>
   </td>
  </tr>
  <tr>
   <td>
    Less Than
   </td>
   <td>
    <code>
     {key: {$lte: value}}
    </code>
   </td>
   <td>
    <code>
     where key &lt; value
    </code>
   </td>
  </tr>
  <tr>
   <td>
    Less Than or Equal
   </td>
   <td>
    <code>
     {key: {$lt: value}}
    </code>
   </td>
   <td>
    <code>
     where key &lt;= value
    </code>
   </td>
  </tr>
  <tr>
   <td>
    Greater Than
   </td>
   <td>
    <code>
     {key: {$gt: value}}
    </code>
   </td>
   <td>
    <code>
     where key &gt; value
    </code>
   </td>
  </tr>
  <tr>
   <td>
    Greater Than or Equal
   </td>
   <td>
    <code>
     {key: {$gte: value}}
    </code>
   </td>
   <td>
    <code>
     where key &gt;= value
    </code>
   </td>
  </tr>
  <tr>
   <td>
    Not Equal
   </td>
   <td>
    <code>
     {key: {$ne: value}}
    </code>
   </td>
   <td>
    <code>
     where key != value
    </code>
   </td>
  </tr>
  <tr>
   <td>
    And
   </td>
   <td>
    <code>
     {$and: [{k1: v1}, {k2: v2}]}
    </code>
   </td>
   <td>
    <code>
     where k1 = v1 and k2 = v2
    </code>
   </td>
  </tr>
  <tr>
   <td>
    Or
   </td>
   <td>
    <code>
     {$or: [{k1: v1}, {k2: v2}]}
    </code>
   </td>
   <td>
    <code>
     where k1 = v1 or k2 = v2
    </code>
   </td>
  </tr>
 </tbody>
</table>

The operations all do what you would expect them to do, and their names are really intuitive. Aggregating operations with `$and` or `$or` can be done anywhere in the query, nested any level deep.

## Update

Updating a document can be done by using [`save`](https://docs.mongodb.com/manual/reference/method/db.collection.save/index.html) on an already-existing document (that is, the document we want to save has `_id` and it’s in the collection already). If the document is not in the collection yet, this method will create it.

```
> db.greetings.save({_id: ObjectId("5e74829a0659f802b15f18dd"), message: "¡Bienvenido, humano!", "lang" : "es"})
WriteResult({ "nMatched" : 1, "nUpserted" : 0, "nModified" : 1 })

> db.greetings.find({lang: "es"})
{ "_id" : ObjectId("5e74829a0659f802b15f18dd"), "message" : "¡Bienvenido, humano!", "lang" : "es" }
```

Alternatively, the [`update`](https://docs.mongodb.com/manual/reference/method/db.collection.update/index.html) method takes a query and new value.

```
> db.greetings.update({lang: "en"}, {$set: {message: "Welcome, human!"}})
WriteResult({ "nMatched" : 1, "nUpserted" : 0, "nModified" : 1 })

> db.greetings.find({lang: "en"})
{ "_id" : ObjectId("5e7487b90659f802b15f18de"), "message" : "Welcome, human!", "lang" : "en" }
```

## Indexing

Creating an index is done with [`createIndex`](https://docs.mongodb.com/manual/reference/method/db.collection.createIndex/index.html):

```
> db.greetings.createIndex({lang: +1})
{
    "createdCollectionAutomatically" : false,
    "numIndexesBefore" : 1,
    "numIndexesAfter" : 2,
    "ok" : 1
}
```

Here, we create an ascending index on the lang key. Descending order is done with `-1`. Now a query for `lang` in our three documents will be fast… well maybe iteration over three documents was faster than an index.

## Delete

### Delete a document

I have to confess, I can’t talk French. I learnt it long ago and it’s long forgotten, so let’s remove the translation I copied online from our greetings with [`remove`](https://docs.mongodb.com/manual/reference/method/db.collection.remove/index.html).

```
> db.greetings.remove({lang: "fr"})
WriteResult({ "nRemoved" : 1 })
```

### Delete a collection

We never really used the `goodbyes` collection. Can we get rid of that?

```
> db.goodbyes.drop()
true
```

Yes, it is `true` that we can [`drop`](https://docs.mongodb.com/manual/reference/method/db.collection.drop/index.html) it.

### Delete a database

Now, I will be honest, I don’t really like our `greetings` database either. It stinks. Let’s get rid of it as well:

```
> db.dropDatabase()
{ "dropped" : "helloworld", "ok" : 1 }
```

Yeah, take that! The [`dropDatabase`](https://docs.mongodb.com/manual/reference/method/db.dropDatabase/) can be used to drop databases.

## References

The examples in this post are all fictional, and the methods that could be used where taken from Classmate’s post, and of course [Mongo’s documentation](https://docs.mongodb.com/manual/reference/method/).