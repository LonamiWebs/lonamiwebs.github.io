+++
title = "About Boolean Retrieval"
date = 2020-02-25T00:00:29+00:00
updated = 2020-03-18T09:38:02+00:00
+++

This entry will discuss the section on the _[Boolean retrieval](https://nlp.stanford.edu/IR-book/pdf/01bool.pdf)_ section of the book _[An Introduction to Information Retrieval](https://nlp.stanford.edu/IR-book/pdf/irbookprint.pdf)_.

## Summary on the topic

Boolean retrieval is one of the many ways information retrieval (finding materials that satisfy an information need), often simply called _search_.

A simple way to retrieve information is to _grep_ through the text (term named after the Unix tool `grep`), scanning text linearly and excluding it on certain criteria. However, this falls short when the volume of the data grows, more complex queries are desired, or one seeks some sort of ranking.

To avoid linear scanning, we build an _index_ and record for each document whether it contains each term out of our full dictionary of terms (which may be words in a chapter and words in the book). This results in a binary term-document _incidence matrix_. Such a possible matrix is:

<table class="">
 <tbody>
  <tr>
   <td>
    <em>
     word/play
    </em>
   </td>
   <td>
    <strong>
     Antony and Cleopatra
    </strong>
   </td>
   <td>
    <strong>
     Julius Caesar
    </strong>
   </td>
   <td>
    <strong>
     The Tempest
    </strong>
   </td>
   <td>
    <strong>
     …
    </strong>
   </td>
  </tr>
  <tr>
   <td>
    <strong>
     Antony
    </strong>
   </td>
   <td>
    1
   </td>
   <td>
    1
   </td>
   <td>
    0
   </td>
   <td>
   </td>
  </tr>
  <tr>
   <td>
    <strong>
     Brutus
    </strong>
   </td>
   <td>
    1
   </td>
   <td>
    1
   </td>
   <td>
    0
   </td>
   <td>
   </td>
  </tr>
  <tr>
   <td>
    <strong>
     Caesar
    </strong>
   </td>
   <td>
    1
   </td>
   <td>
    1
   </td>
   <td>
    0
   </td>
   <td>
   </td>
  </tr>
  <tr>
   <td>
    <strong>
     Calpurnia
    </strong>
   </td>
   <td>
    0
   </td>
   <td>
    1
   </td>
   <td>
    0
   </td>
   <td>
   </td>
  </tr>
  <tr>
   <td>
    <strong>
     Cleopatra
    </strong>
   </td>
   <td>
    1
   </td>
   <td>
    0
   </td>
   <td>
    0
   </td>
   <td>
   </td>
  </tr>
  <tr>
   <td>
    <strong>
     mercy
    </strong>
   </td>
   <td>
    1
   </td>
   <td>
    0
   </td>
   <td>
    1
   </td>
   <td>
   </td>
  </tr>
  <tr>
   <td>
    <strong>
     worser
    </strong>
   </td>
   <td>
    1
   </td>
   <td>
    0
   </td>
   <td>
    1
   </td>
   <td>
   </td>
  </tr>
  <tr>
   <td>
    <strong>
     …
    </strong>
   </td>
   <td>
   </td>
   <td>
   </td>
   <td>
   </td>
   <td>
   </td>
  </tr>
 </tbody>
</table>

We can look at this matrix’s rows or columns to obtain a vector for each term indicating where it appears, or a vector for each document indicating the terms it contains.

Now, answering a query such as `Brutus AND Caesar AND NOT Calpurnia` becomes trivial:

```
VECTOR(Brutus) AND VECTOR(Caesar) AND COMPLEMENT(VECTOR(Calpurnia))
= 110 AND 110 AND COMPLEMENT(010)
= 110 AND 110 AND 101
= 100
```

The query is only satisfied for our first column.

The _Boolean retrieval model_ is thus a model that treats documents as a set of terms, in which we can perform any query in the form of Boolean expressions of terms, combined with `OR`, `AND`, and `NOT`.

Now, building such a matrix is often not feasible due to the sheer amount of data (say, a matrix with 500,000 terms across 1,000,000 documents, each with roughly 1,000 terms). However, it is important to notice that most of the terms will be _missing_ when examining each document. In our example, this means 99.8% or more of the cells will be 0. We can instead record the _positions_ of the 1’s. This is known as an _inverted index_.

The inverted index is a dictionary of terms, each containing a list that records in which documents it appears (_postings_). Applied to boolean retrieval, we would:

1. Collects the documents to be indexed, assign a unique identifier each
2. Tokenize the text in the documents into a list of terms
3. Normalize the tokens, which now become indexing terms
4. Index the documents

<table class="">
 <tbody>
  <tr>
   <td>
    <strong>
     Dictionary
    </strong>
   </td>
   <td>
    <strong>
     Postings
    </strong>
   </td>
  </tr>
  <tr>
   <td>
    Brutus
   </td>
   <td>
    1, 2, 4, 11, 31, 45, 173, 174
   </td>
  </tr>
  <tr>
   <td>
    Caesar
   </td>
   <td>
    1, 2, 4, 5, 6, 16, 57, 132, …
   </td>
  </tr>
  <tr>
   <td>
    Calpurnia
   </td>
   <td>
    2, 31, 54, 101
   </td>
  </tr>
  <tr>
   <td>
    …
   </td>
   <td>
   </td>
  </tr>
 </tbody>
</table>

Sort the pairs `(term, document_id)` so that the terms are alphabetical, and merge multiple occurences into one. Group instances of the same term and split again into a sorted list of postings.

<table class="">
 <tbody>
  <tr>
   <td>
    <strong>
     term
    </strong>
   </td>
   <td>
    <strong>
     document_id
    </strong>
   </td>
  </tr>
  <tr>
   <td>
    I
   </td>
   <td>
    1
   </td>
  </tr>
  <tr>
   <td>
    did
   </td>
   <td>
    1
   </td>
  </tr>
  <tr>
   <td>
    …
   </td>
   <td>
   </td>
  </tr>
  <tr>
   <td>
    with
   </td>
   <td>
    2
   </td>
  </tr>
 </tbody>
</table>

<table class="">
 <tbody>
  <tr>
   <td>
    <strong>
     term
    </strong>
   </td>
   <td>
    <strong>
     document_id
    </strong>
   </td>
  </tr>
  <tr>
   <td>
    be
   </td>
   <td>
    2
   </td>
  </tr>
  <tr>
   <td>
    brutus
   </td>
   <td>
    1
   </td>
  </tr>
  <tr>
   <td>
    brutus
   </td>
   <td>
    2
   </td>
  </tr>
  <tr>
   <td>
    …
   </td>
   <td>
   </td>
  </tr>
 </tbody>
</table>

<table class="">
 <tbody>
  <tr>
   <td>
    <strong>
     term
    </strong>
   </td>
   <td>
    <strong>
     frequency
    </strong>
   </td>
   <td>
    <strong>
     postings list
    </strong>
   </td>
  </tr>
  <tr>
   <td>
    be
   </td>
   <td>
    1
   </td>
   <td>
    2
   </td>
  </tr>
  <tr>
   <td>
    brutus
   </td>
   <td>
    2
   </td>
   <td>
    1, 2
   </td>
  </tr>
  <tr>
   <td>
    capitol
   </td>
   <td>
    1
   </td>
   <td>
    1
   </td>
  </tr>
  <tr>
   <td>
    …
   </td>
   <td>
   </td>
   <td>
   </td>
  </tr>
 </tbody>
</table>

Intersecting posting lists now becomes of transversing both lists in order:

```
Brutus   : 1 -> 2 -> 4 -> 11 -> 31 -> 45 ->           173 -> 174
Calpurnia:      2 ->            31 ->       54 -> 101
Intersect:      2 ->            31
```

A simple conjunctive query (e.g. `Brutus AND Calpurnia`) is executed as follows:

1. Locate `Brutus` in the dictionary
2. Retrieve its postings
3. Locate `Calpurnia` in the dictionary
4. Retrieve its postings
5. Intersect (_merge_) both postings

Since the lists are sorted, walking both of them can be done in _O(n)_ time. By also storing the frequency, we can optimize the order in which we execute arbitrary queries, although we won’t go into detail.

## Thoughts

The boolean retrieval model can be implemented with relative ease, and can help with storage and efficient querying of the information if we intend to perform boolean queries.

However, the basic design lacks other useful operations, such as a «near» operator, or the ability to rank the results.

All in all, it’s an interesting way to look at the data and query it efficiently.