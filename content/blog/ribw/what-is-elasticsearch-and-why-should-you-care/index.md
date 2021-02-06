+++
title = "What is ElasticSearch and why should you care?"
date = 2020-03-18T02:00:00+00:00
updated = 2020-03-27T11:04:45+00:00
+++

ElasticSearch is a giant search index with powerful analytics capabilities. It’s like a database and search engine on steroids, really easy and fast to get up and running. One can think of it as your own Google, a search engine with analytics.

ElasticSearch is rich, stable, performs well, is well maintained, and able to scale to petabytes of any kind of data, whether it’s structured, semi-structured or not at all. It’s cost-effective and can be used to make business decisions.

Or, described in 10 seconds:

> Schema-free, REST & JSON based distributed document store
> Open source: Apache License 2.0
> Zero configuration

-- Alex Reelsen

## Basic capabilities

ElasticSearch lets you ask questions about your data, not just make queries. You may think SQL can do this too, but what’s important is making a pipeline of facets, and feed the results from query to query.

Instead of changing your data, you can be flexible with your questions with no need to re-index it every time the questions change.

ElasticSearch is not just to search for full-text data, either. It can search for structured data and return more than just the results. It also yields additional data, such as ranking, highlights, and allows for pagination.

It doesn’t take a lot of configuration to get running, either, which can be a good boost on productivity.

## How does it work?

ElasticSearch depends on Java, and can work in a distributed cluster if you execute multiple instances. Data will be replicated and sharded as needed. The current version at the time of writing is 7.6.1, and it’s being developed fast!

It also has support for plugins, with an ever-growing ecosystem and integration on many programming languages. Tools around it are being built around it, too, like Kibana which helps you visualize your data.

The way you use it is through a JSON API, served over HTTP/S.

## How can I use it?

[You can try ElasticSearch out for free on Elastic Cloud](https://www.elastic.co/downloads/), however, it can also be [downloaded and ran offline](https://www.elastic.co/downloads/elasticsearch), which is what we’ll do. Download the file corresponding to your operating system, unzip it, and execute the binary. Running it is as simple as that!

Now you can make queries to it over HTTP, with for example `curl`:

```
curl -X PUT localhost:9200/orders/order/1 -d '
{
  "created_at": "2013/09/05 15:45:10",
  "items": [
    {
      name: "HD Monitor"
    }
  ],
  "total": 249.95
}'
```

This will create a new order with some information, such as when it was created, what items it contains, and the total cost of the order.

You can then query or filter as needed, script it or even create statistics.

## References

* [YouTube – What is Elasticsearch?](https://youtu.be/sKnkQSec1U0)
* [YouTube – GOTO 2013 • Elasticsearch – Beyond Full-text Search • Alex Reelsen](https://youtu.be/yWNiRC_hUAw)
* [Kibana – Your window into the Elastic Stack](https://www.elastic.co/kibana)
* [Elastic Stack and Product Documentation](https://www.elastic.co/guide/index.html)