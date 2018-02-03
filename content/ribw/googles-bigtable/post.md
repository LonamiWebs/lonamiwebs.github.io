```meta
title: Google’s BigTable
published: 2020-04-01T00:00:00+00:00
updated: 2020-04-03T09:30:05+00:00
```

Let’s talk about BigTable, and why it is what it is. But before we get into that, let’s see some important aspects anybody should consider when dealing with a lot of data (something BigTable does!).

## The basics

Converting a text document into a different format is often a great way to greatly speed up scanning of it in the future. It allows for efficient searches.

In addition, you generally want to store everything in a single, giant file. This will save a lot of time opening and closing files, because everything is in the same file! One proposal to make this happen is [Web TREC](https://trec.nist.gov/file_help.html) (see also the [Wikipedia page on TREC](https://en.wikipedia.org/wiki/Text_Retrieval_Conference)), which is basically HTML but every document is properly delimited from one another.

Because we will have a lot of data, it’s often a good idea to compress it. Most text consists of the same words, over and over again. Classic compression techniques such as `DEFLATE` or `LZW` do an excellent job here.

## So what’s BigTable?

Okay, enough of an introduction to the basics on storing data. BigTable is what Google uses to store documents, and it’s a customized approach to save, search and update web pages.

BigTable is is a distributed storage system for managing structured data, able to scale to petabytes of data across thousands of commodity servers, with wide applicability, scalability, high performance, and high availability.

In a way, it’s kind of like databases and shares many implementation strategies with them, like parallel databases, or main-memory databases, but of course, with a different schema.

It consists of a big table known as the «Root tablet», with pointers to many other «tablets» (or metadata in between). These are stored in a replicated filesystem accessible by all BigTable servers. Any change to a tablet gets logged (said log also gets stored in a replicated filesystem).

If any of the tablets servers gets locked, a different one can take its place, read the log and deal with the problem.

There’s no query language, transactions occur at row-level only. Every read or write in a row is atomic. Each row stores a single web page, and by combining the row and column keys along with a timestamp, it is possible to retrieve a single cell in the row. More formally, it’s a map that looks like this:

```
fetch(row: string, column: string, time: int64) -> string
```

A row may have as many columns as it needs, and these column groups are the same for everyone (but the columns themselves may vary), which is importan to reduce disk read time.

Rows are split in different tablets based on the row keys, which simplifies determining an appropriated server for them. The keys can be up to 64KB big, although most commonly they range 10-100 bytes.

## Conclusions

BigTable is Google’s way to deal with large amounts of data on many of their services, and the ideas behind it are not too complex to understand.
