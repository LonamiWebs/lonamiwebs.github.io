+++
title = "A practical example with Hadoop"
date = 2020-04-01T02:00:00+00:00
updated = 2020-04-03T08:43:41+00:00
+++

In our [previous Hadoop post](/blog/ribw/introduction-to-hadoop-and-its-mapreduce/), we learnt what it is, how it originated, and how it works, from a theoretical standpoint. Here we will instead focus on a more practical example with Hadoop.

This post will showcase my own implementation to implement a word counter for any plain text document that you want to analyze.

## Installation

Before running any piece of software, its executable code must first be downloaded into our computers so that we can run it. Head over to [Apache Hadoop’s releases](http://hadoop.apache.org/releases.html) and download the [latest binary version](https://www.apache.org/dyn/closer.cgi/hadoop/common/hadoop-3.2.1/hadoop-3.2.1.tar.gz) at the time of writing (3.2.1).

We will be using the [Linux Mint](https://linuxmint.com/) distribution because I love its simplicity, although the process shown here should work just fine on any similar Linux distribution such as [Ubuntu](https://ubuntu.com/).

Once the archive download is complete, extract it with any tool of your choice (graphical or using the terminal) and execute it. Make sure you have a version of Java installed, such as [OpenJDK](https://openjdk.java.net/).

Here are all the three steps in the command line:

```
wget https://www.apache.org/dyn/closer.cgi/hadoop/common/hadoop-3.2.1/hadoop-3.2.1.tar.gz
tar xf hadoop-3.2.1.tar.gz
hadoop-3.2.1/bin/hadoop version
```

## Processing data

To take advantage of Hadoop, we have to design our code to work in the MapReduce model. Both the map and reduce phase work on key-value pairs as input and output, and both have a programmer-defined function.

We will use Java, because it’s a dependency that we already have anyway, so might as well.

Our map function needs to split each of the lines we receive as input into words, and we will also convert them to lowercase, thus preparing the data for later use (counting words). There won’t be bad records, so we don’t have to worry about that.

Copy or reproduce the following code in a file called `WordCountMapper.java`, using any text editor of your choice:

```
import java.io.IOException;

import org.apache.hadoop.io.IntWritable;
import org.apache.hadoop.io.LongWritable;
import org.apache.hadoop.io.Text;
import org.apache.hadoop.mapreduce.Mapper;

public class WordCountMapper extends Mapper<LongWritable, Text, Text, IntWritable> {
    @Override
    public void map(LongWritable key, Text value, Context context)
            throws IOException, InterruptedException {
        for (String word : value.toString().split("\\W")) {
            context.write(new Text(word.toLowerCase()), new IntWritable(1));
        }
    }
}
```

Now, let’s create the `WordCountReducer.java` file. Its job is to reduce the data from multiple values into just one. We do that by summing all the values (our word count so far):

```
import java.io.IOException;
import java.util.Iterator;

import org.apache.hadoop.io.IntWritable;
import org.apache.hadoop.io.Text;
import org.apache.hadoop.mapreduce.Reducer;

public class WordCountReducer extends Reducer<Text, IntWritable, Text, IntWritable> {
    @Override
    public void reduce(Text key, Iterable<IntWritable> values, Context context)
            throws IOException, InterruptedException {
        int count = 0;
        for (IntWritable value : values) {
            count += value.get();
        }
        context.write(key, new IntWritable(count));
    }
}
```

Let’s just take a moment to appreciate how absolutely tiny this code is, and it’s Java! Hadoop’s API is really awesome and lets us write such concise code to achieve what we need.

Last, let’s write the `main` method, or else we won’t be able to run it. In our new file `WordCount.java`:

```
import org.apache.hadoop.fs.Path;
import org.apache.hadoop.io.IntWritable;
import org.apache.hadoop.io.Text;
import org.apache.hadoop.mapreduce.Job;
import org.apache.hadoop.mapreduce.lib.input.FileInputFormat;
import org.apache.hadoop.mapreduce.lib.output.FileOutputFormat;

public class WordCount {
    public static void main(String[] args) throws Exception {
        if (args.length != 2) {
            System.err.println("usage: java WordCount <input path> <output path>");
            System.exit(-1);
        }

        Job job = Job.getInstance();

        job.setJobName("Word count");
        job.setJarByClass(WordCount.class);
        job.setMapperClass(WordCountMapper.class);
        job.setReducerClass(WordCountReducer.class);
        job.setOutputKeyClass(Text.class);
        job.setOutputValueClass(IntWritable.class);

        FileInputFormat.addInputPath(job, new Path(args[0]));
        FileOutputFormat.setOutputPath(job, new Path(args[1]));

        boolean result = job.waitForCompletion(true);

        System.exit(result ? 0 : 1);
    }
}
```

And compile by including the required `.jar` dependencies in Java’s classpath with the `-cp` switch:

```
javac -cp "hadoop-3.2.1/share/hadoop/common/*:hadoop-3.2.1/share/hadoop/mapreduce/*" *.java
```

At last, we can run it (also specifying the dependencies in the classpath, this one’s a mouthful). Let’s run it on the same `WordCount.java` source file we wrote:

```
java -cp ".:hadoop-3.2.1/share/hadoop/common/*:hadoop-3.2.1/share/hadoop/common/lib/*:hadoop-3.2.1/share/hadoop/mapreduce/*:hadoop-3.2.1/share/hadoop/mapreduce/lib/*:hadoop-3.2.1/share/hadoop/yarn/*:hadoop-3.2.1/share/hadoop/yarn/lib/*:hadoop-3.2.1/share/hadoop/hdfs/*:hadoop-3.2.1/share/hadoop/hdfs/lib/*" WordCount WordCount.java results
```

Hooray! We should have a new `results/` folder along with the following files:

```
$ ls results
part-r-00000  _SUCCESS
$ cat results/part-r-00000
	154
0	2
1	3
2	1
addinputpath	1
apache	6
args	4
boolean	1
class	6
count	1
err	1
exception	1
-snip- (output cut for clarity)
```

It worked! Now this example was obviously tiny, but hopefully enough to demonstrate how to get the basics running on real world data.