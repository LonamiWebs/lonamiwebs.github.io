```meta
title: Big Data
published: 2020-02-25T01:00:30+00:00
updated: 2020-03-18T09:51:17+00:00
```

Big Data sounds like a buzzword you may be hearing everywhere, but it’s actually here to stay!

## What is Big Data?

And why is it so important? We use this term to refer to the large amount of data available, rapidly growing every day, that cannot be processed in conventional ways. It’s not only about the amount, it’s also about the variety and rate of growth.

Thanks to technological advancements, there are new ways to process this insane amount of data, which would otherwise be too costly for processing in traditional database systems.

## Where does data come from?

It can be pictures in your phone, industry transactions, messages in social networks, a sensor in the mountains. It can come from anywhere, which makes the data very varied.

Just to give some numbers, over 12TB of data is generated on Twitter _daily_. If you purchase a laptop today (as of March 2020), the disk will be roughly 1TB, maybe 2TB. Twitter would fill 6 of those drives every day!

What about Facebook? It is estimated they store around 100PB of photos and videos. That would be 50000 laptop disks. Not a small number. And let’s not talk about worldwide network traffic…

## What data can be exploited?

So, we have a lot of data. Should we attempt and process everything? We can distinguish several categories.

* **Web and Social Media**: Clickstream Data, Twitter Feeds, Facebook Postings, Web content… Stuff coming from social networks.
* **Biometrics**: Facial Recognion, Genetics… Any kind of personal recognition.
* **Machine-to-Machine**: Utility Smart Meter Readings, RFID Readings, Oil Rig Sensor Readings, GPS Signals… Any sensor shared with other machines.
* **Human Generated**: Call Center Voice Recordings, Email, Electronic Medical Records… Even the voice notes one sends over WhatsApp count.
* **Big Transaction Data**: Healthcare Claims, Telecommunications Call Detail Records, Utility Billing Records… Financial transactions.

But asking what to process is asking the wrong question. Instead, one should think about «What problem am I trying to solve?».

## How to exploit this data?

What are some of the ways to deal with this data? If the problem fits the Map-Reduce paradigm then Hadoop is a great option! Hadoop is inspired by Google File System (GFS), and achieves great parallelism across the nodes of a cluster, and has the following components:

* **Hadoop Distributed File System**. Data is divided into smaller «blocks» and distributed across the cluster, which makes it possible to execute the mapping and reduction in smaller subsets, and makes it possible to scale horizontally.
* **Hadoop MapReduce**. First, a data set is «mapped» into a different set, and data becomes a list of tuples (key, value). The «reduce» step works on these tuples and combines them into a smaller subset.
* **Hadoop Common**. These are a set of libraries that ease working with Hadoop.

## Key insights

Big Data is a field whose goal is to extract information from very large sets of data, and find ways to do so. To summarize its different dimensions, we can refer to what’s known as «the Four V’s of Big Data»:

* **Volume**. Really large quantities.
* **Velocity**. Processing response time matters!
* **Variety**. Data comes from plenty of sources.
* **Veracity.** Can we trust all sources, though?

Some sources talk about a fifth V for **Value**; because processing this data is costly, it is important we can get value out of it.

…And some other sources go as high as seven V’s, including **Viability** and **Visualization**. Computers can’t take decissions on their own (yet), a human has to. And they can only do so if they’re presented the data (and visualize it) in a meaningful way.

## Infographics

Let’s see some pictures, we all love pictures:

![](4-Vs-of-big-data.jpg)

## Common patterns

## References

* ¿Qué es Big Data? – [https://www.ibm.com/developerworks/ssa/local/im/que-es-big-data/](https://www.ibm.com/developerworks/ssa/local/im/que-es-big-data/)
* The Four V’s of Big Data – [https://www.ibmbigdatahub.com/infographic/four-vs-big-data](https://www.ibmbigdatahub.com/infographic/four-vs-big-data)
* Big data – [https://en.wikipedia.org/wiki/Big_data](https://en.wikipedia.org/wiki/Big_data)
* Las 5 V’s del Big Data – [https://www.quanticsolutions.es/big-data/las-5-vs-del-big-data](https://www.quanticsolutions.es/big-data/las-5-vs-del-big-data)
* Las 7 V del Big data: Características más importantes – [https://www.iic.uam.es/innovacion/big-data-caracteristicas-mas-importantes-7-v/](https://www.iic.uam.es/innovacion/big-data-caracteristicas-mas-importantes-7-v/#viabilidad)
