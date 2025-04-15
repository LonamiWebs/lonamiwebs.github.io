+++
title = "Mining of Massive Datasets"
date = 2020-03-16T01:00:00+00:00
updated = 2020-03-28T19:09:44+00:00
+++

In this post we will talk about the Chapter 1 of the book Mining of Massive Datasets Leskovec, J. et al., available online, and I will summarize and share my thoughts on it.

Data mining often refers to the discovery of models for data, where the model can be for statistics, machine learning, summarizing, extracting features, or other computational approaches to perform complex queries on the data.

Commonly, problems related to data mining involve discovering unusual events hidden in massive data sets. There is another problem when trying to achieve Total Information Awareness (TIA), though, a project that was proposed by the Bush administration but shut down. The problem is, if you look at so much data, and try to find activities that look like (for example) terrorist behavior, inevitably one will find other illicit activities that are not terrorism with bad consequences. So it is important to narrow the activities we are looking for, in this case.

When looking at data, even completely random data, for a certain event type, the event will likely occur. With more data, it will occur more times. However, these are bogus results. The Bonferroni correction gives a statistically sound way to avoid most of these bogus results, however, the Bonferroni’s Principle can be used as an informal version to achieve the same thing.

For that, we calculate the expected number of occurrences of the events you are looking for on the assumption that data is random. If this number is way larger than the number of real instances one hoped to find, then nearly everything will be Bogus.

----------

When analysing documents, some words will be more important than others, and can help determine the topic of the document. One could think the most repeated words are the most important, but that’s far from the truth. The most common words are the stop-words, which carry no meaning, reason why we should remove them prior to processing. We are mostly looking for rare nouns.

There are of course formal measures for how concentrated into relatively few documents are the occurrences of a given word, known as TF.IDF (Term Frequency times In-verse Document Frequency). We won’t go into details on how to compute it, because there are multiple ways.

Hash functions are also frequently used, because they can turn hash keys into a bucket number (the index of the bucket where this hash key belongs). They «randomize» and spread the universe of keys into a smaller number of buckets, useful for storage and access.

An index is an efficient structure to query for values given a key, and can be built with hash functions and buckets.

Having all of these is important when analysing documents when doing data mining, because otherwise it would take far too long.