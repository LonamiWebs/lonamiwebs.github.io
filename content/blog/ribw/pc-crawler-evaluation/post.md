```meta
title: Privado: PC-Crawler evaluation
published: 2020-03-04T00:00:23+00:00
updated: 2020-03-18T09:39:27+00:00
```

As the student `a(i)` where `i = 9`, I have been assigned to evaluate students `a(i + 3)` and `a(i + 4)`, these being:

* a12: Classmate (username)
* a13: Classmate (username)

## Classmate’s evaluation

**Grading: B.**

I think they mix up a bit their considerations with program usage and how it works, not justifying why the considerations are the ones they chose, or what the alternatives would be.

The implementation notes are quite well-written. Even someone without knowledge of Java’s syntax can read the notes and more or less make sense of what’s going on, with the relevant code excerpts on each section.

Implementation-wise, some methods could definitely use some improvement:

* `esExtensionTextual` is overly complicated. It could use a `for` loop and Java’s `String.endsWith`.
* `calcularFrecuencia` has quite some duplication (e.g. `this.getFicherosYDirectorios().remove(0)`) and could definitely be cleaned up.

However, all the desired functionality is implemented.

Style-wise, some of the newlines and avoiding braces on `if` and `while` could be changed to improve the readability.

The post is written in Spanish, but uses some words that don’t translate well («remover» could better be said as «eliminar» or «quitar»).

## Classmate’s evaluation

**Grading: B.**

Their post starts with an explanation on what a crawler is, common uses for them, and what type of crawler they will be developing. This is a very good start. Regarding the post style, it seems they are not properly using some of WordPress features, such as lists, and instead rely on paragraphs with special characters prefixing each list item.

The post also contains some details on how to install the requirements to run the program, which can be very useful for someone not used to working with Java.

They do not explain their implementation and the filename of the download has a typo.

Implementation-wise, the code seems to be well-organized, into several packages and files, although the naming is a bit inconsistent. They even designed a GUI, which is quite impressive.

Some of the methods are documented, although the code inside them is not very commented, including missing rationale for the data structures chosen. There also seem to be several other unused main functions, which I’m unsure why they were kept.

However, all the desired functionality is implemented.

Similar to Classmate, the code style could be improved and settled on some standard, as well as making use of Java features such as `for` loops over iterators instead of manual loops.
