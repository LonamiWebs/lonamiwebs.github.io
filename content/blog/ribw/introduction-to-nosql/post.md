```meta
title: Introduction to NoSQL
published: 2020-02-25T01:00:15+00:00
updated: 2020-03-18T09:38:23+00:00
```

This post will primarly focus on the talk held in the [GOTO 2012 conference: Introduction to NoSQL by Martin Fowler](https://youtu.be/qI_g07C_Q5I). It can be seen as an informal, summarized transcript of the talk

----------

The relational database model is affected by the _[impedance mismatch problem](https://en.wikipedia.org/wiki/Object-relational_impedance_mismatch)_. This occurs because we have to match our high-level design with the separate columns and rows used by relational databases.

Taking the in-memory objects and putting them into a relational database (which were dominant at the time) simply didn’t work out. Why? Relational databases were more than just databases, they served as a an integration mechanism across applications, up to the 2000s. For 20 years!

With the rise of the Internet and the sheer amount of traffic, databases needed to scale. Unfortunately, relational databases only scale well vertically (by upgrading a _single_ node). This is _very_ expensive, and not something many could afford.

The problem are those pesky `JOIN`‘s, and its friends `GROUP BY`. Because our program and reality model don’t match the tables used by SQL, we have to rely on them to query the data. It is because the model doesn’t map directly.

Furthermore, graphs don’t map very well at all to relational models.

We needed a way to scale horizontally (by increasing the _amount_ of nodes), something relational databases were not designed to do.

> _We need to do something different, relational across nodes is an unnatural act_

This inspired the NoSQL movement.

> _#nosql was only meant to be a hashtag to advertise it, but unfortunately it’s how it is called now_

It is not possible to define NoSQL, but we can identify some of its characteristics:

* Non-relational
* **Cluster-friendly** (this was the original spark)
* Open-source (until now, generally)
* 21st century web culture
* Schema-less (easier integration or conjugation of several models, structure aggregation)
These databases use different data models to those used by the relational model. However, it is possible to identify 4 broad chunks (some may say 3, or even 2!):

* **Key-value store**. With a certain key, you obtain the value corresponding to it. It knows nothing else, nor does it care. We say the data is opaque.
* **Document-based**. It stores an entire mass of documents with complex structure, normally through the use of JSON (XML has been left behind). Then, you can ask for certain fields, structures, or portions. We say the data is transparent.
* **Column-family**. There is a «row key», and within it we store multiple «column families» (columns that fit together, our aggregate). We access by row-key and column-family name.
All of these kind of serve to store documents without any _explicit_ schema. Just shove in anything! This gives a lot of flexibility and ease of migration, except… that’s not really true. There’s an _implicit_ schema when querying.

For example, a query where we may do `anOrder['price'] * anOrder['quantity']` is assuming that `anOrder` has both a `price` and a `quantity`, and that both of these can be multiplied together. «Schema-less» is a fuzzy term.

However, it is the lack of a _fixed_ schema that gives flexibility.

One could argue that the line between key-value and document-based is very fuzzy, and they would be right! Key-value databases often let you include additional metadata that behaves like an index, and in document-based, documents often have an identifier anyway.

The common notion between these three types is what matters. They save an entire structure as an _unit_. We can refer to these as «Aggregate Oriented Databases». Aggregate, because we group things when designing or modeling our systems, as opposed to relational databases that scatter the information across many tables.

There exists a notable outlier, though, and that’s:

* **Graph** databases. They use a node-and-arc graph structure. They are great for moving on relationships across things. Ironically, relational databases are not very good at jumping across relationships! It is possibly to perform very interesting queries in graph databases which would be really hard and costly on relational models. Unlike the aggregated databases, graphs break things into even smaller units.
NoSQL is not _the_ solution. It depends on how you’ll work with your data. Do you need an aggregate database? Will you have a lot of relationships? Or would the relational model be good fit for you?

NoSQL, however, is a good fit for large-scale projects (data will _always_ grow) and faster development (the impedance mismatch is drastically reduced).

Regardless of our choice, it is important to remember that NoSQL is a young technology, which is still evolving really fast (SQL has been stable for _decades_). But the _polyglot persistence_ is what matters. One must know the alternatives, and be able to choose.

----------

Relational databases have the well-known ACID properties: Atomicity, Consistency, Isolation and Durability.

NoSQL (except graph-based!) are about being BASE instead: Basically Available, Soft state, Eventual consistency.

SQL needs transactions because we don’t want to perform a read while we’re only half-way done with a write! The readers and writers are the problem, and ensuring consistency results in a performance hit, even if the risk is low (two writers are extremely rare but it still must be handled).

NoSQL on the other hand doesn’t need ACID because the aggregate _is_ the transaction boundary. Even before NoSQL itself existed! Any update is atomic by nature. When updating many documents it _is_ a problem, but this is very rare.

We have to distinguish between logical and replication consistency. During an update and if a conflict occurs, it must be resolved to preserve the logical consistency. Replication consistency on the other hand is preserveed when distributing the data across many machines, for example during sharding or copies.

Replication buys us more processing power and resillence (at the cost of more storage) in case some of the nodes die. But what happens if what dies is the communication across the nodes? We could drop the requests and preserve the consistency, or accept the risk to continue and instead preserve the availability.

The choice on whether trading consistency for availability is acceptable or not depends on the domain rules. It is the domain’s choice, the business people will choose. If you’re Amazon, you always want to be able to sell, but if you’re a bank, you probably don’t want your clients to have negative numbers in their account!

Regardless of what we do, in a distributed system, the CAP theorem always applies: Consistecy, Availability, Partitioning-tolerancy (error tolerancy). It is **impossible** to guarantee all 3 at 100%. Most of the times, it does work, but it is mathematically impossible to guarantee at 100%.

A database has to choose what to give up at some point. When designing a distributed system, this must be considered. Normally, the choice is made between consistency or response time.

## Further reading

* [The future is: ~~NoSQL Databases~~ Polyglot Persistence](https://www.martinfowler.com/articles/nosql-intro-original.pdf)
* [NoSQL Databases: An Overview](https://www.thoughtworks.com/insights/blog/nosql-databases-overview)
