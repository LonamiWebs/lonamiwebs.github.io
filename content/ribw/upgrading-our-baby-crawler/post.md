```meta
title: Upgrading our Baby Crawler
published: 2020-03-11T00:00:07+00:00
updated: 2020-03-18T09:49:33+00:00
```

In our [last post on this series](/blog/ribw/build-your-own-pc/), we presented the code for our Personal Crawler. However, we didn’t quite explain what a crawler even is! We will use this moment to go a bit more in-depth, and make some upgrades to it.

## What is a Crawler?

A crawler is a program whose job is to analyze documents and extract data from them. For example, search engines like [DuckDuckGo](http://duckduckgo.com/), [Bing](https://bing.com/) or [Google](http://google.com/) all have crawlers to analyze websites and build a database around them. They are some kind of «trackers», because they keep track of everything they find.

Their basic behaviour can be described as follows: given a starting list of URLs, follow them all and identify hyperlinks inside the documents. Add these to the list of links to follow, and repeat _ad infinitum_.

* This lets us create an index to quickly search across them all.
* We can also identify broken links.
* We can gather any other type of information that we found.
Our crawler will work offline, within our own computer, scanning the text documents it finds on the root we tell it to scan.

## Design Decissions

* We will use Java. Its runtime is quite ubiquitous, so it should be able to run in virtually anywhere. The language is typed, which helps catch errors early on.
* Our solution is iterative. While recursion can be seen as more elegants by some, iterative solutions are often more performant with less need for optimization.

## Requirements

If you don’t have Java installed yet, you can [Download Free Java Software](https://java.com/en/download/) from Oracle’s site. To compile the code, the [Java Development Kit](https://www.oracle.com/java/technologies/javase-jdk8-downloads.html) is also necessary.

We don’t depend on any other external libraries, for easier deployment and compilation.

## Implementation

Because the code was getting pretty large, it has been split into several files, and we have also upgraded it to use a Graphical User Interface instead! We decided to use Swing, based on the Java tutorial [Creating a GUI With JFC/Swing](https://docs.oracle.com/javase/tutorial/uiswing/).

### App

This file is the entry point of our application. Its job is to initialize the components, lay them out in the main panel, and connect the event handlers.

Most widgets are pretty standard, and are defined as class variables. However, some variables are notable. The `[DefaultTableModel](https://docs.oracle.com/javase/8/docs/api/javax/swing/table/DefaultTableModel.html)` is used because it allows to [dynamically add rows](https://stackoverflow.com/a/22550106), and we also have a `[SwingWorker](https://docs.oracle.com/javase/8/docs/api/javax/swing/SwingWorker.html)` subclass responsible for performing the word analysis (which is quite CPU intensive and should not be ran in the UI thread!).

There’s a few utility methods to ease some common operations, such as `updateStatus` which changes the status label in the main window, informing the user of the latest changes.

### Thesaurus

A thesaurus is a collection of words or terms used to represent concepts. In literature this is commonly known as a dictionary.

On the subject of this project, we are using a thesaurus based on how relevant is a word for the meaning of a sentence, filtering out those that barely give us any information.

This file contains a simple thesaurus implementation, which can trivially be used as a normal or inverted thesaurus. However, we only treat it as inverted, and its job is loading itself and determining if words are valid or should otherwise be ignored.

### Utils

Several utility functions used across the codebase.

### WordMap

This file is the important one, and its implementation hasn’t changed much since our last post. Instances of a word map contain… wait for it… a map of words! It stores the mapping `word → count` in memory, and offers methods to query the count of a word or iterate over the word count entries.

It can be loaded from cache or told to analyze a root path. Once an instance is created, additional files could be analyzed one by one if desired.

## Download

The code was getting a bit too large to embed it within the blog post itself, so instead you can download it as a`.zip` file.

*download removed*
