```meta
title: Integrating Apache Tika into our Crawler
published: 2020-03-18T00:00:00+00:00
updated: 2020-03-25T17:38:07+00:00
```

[In our last crawler post](/blog/ribw/upgrading-our-baby-crawler/), we detailed how our crawler worked, and although it did a fine job, it’s time for some extra upgrading.

## What kind of upgrades?

A small but useful one. We are adding support for file types that contain text but cannot be processed by normal text editors because they are structured and not just plain text (such as PDF files, Excel, Word documents…).

And for this task, we will make use of the help offered by [Tika](https://tika.apache.org/), our friendly Apache tool.

## What is Tika?

[Tika](https://tika.apache.org/) is a set of libraries offered by [The Apache Software Foundation](https://en.wikipedia.org/wiki/The_Apache_Software_Foundation) that we can include in our project in order to extract the text and metadata of files from a [long list of supported formats](https://tika.apache.org/1.24/formats.html).

## Changes in the code

Not much has changed in the structure of the crawler, we simply have added a new method in `Utils` that uses the class `Tika` from the previously mentioned library so as to process and extract the text of more filetypes.

Then, we use this text just like we would for our standard text file (checking the thesaurus and adding it to the word map) and voilà! We have just added support for a big range of file types.

## Incorporating Gradle

In order for the previous code to work, we need to make use of external libraries. To make this process easier and because the project is growing, we decided to use [Gradle](https://gradle.org/), a build system that can be used for projects in various programming languages, such as Java.

We followed their [guide to Building Java Applications](https://guides.gradle.org/building-java-applications/), and in a few steps added the required `.gradle` files. Now we can compile and run the code without having to worry about juggling with Java and external dependencies in a single command:

```
./gradlew run
```

## Download

And here you can download the final result:

*download removed*
