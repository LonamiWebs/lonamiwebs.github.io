+++
title = "A practical example with Hadoop"
date = 2020-03-30T01:00:00+00:00
updated = 2020-04-18T13:25:43+00:00
+++

In our [previous Hadoop post](/blog/mdad/introduction-to-hadoop-and-its-mapreduce/), we learnt what it is, how it originated, and how it works, from a theoretical standpoint. Here we will instead focus on a more practical example with Hadoop.

This post will reproduce the example on Chapter 2 of the book [Hadoop: The Definitive Guide, Fourth Edition](http://www.hadoopbook.com/) ([pdf,](http://grut-computing.com/HadoopBook.pdf)[code](http://www.hadoopbook.com/code.html)), that is, finding the maximum global-wide temperature for a given year.

## Installation

Before running any piece of software, its executable code must first be downloaded into our computers so that we can run it. Head over to [Apache Hadoop’s releases](http://hadoop.apache.org/releases.html) and download the [latest binary version](https://www.apache.org/dyn/closer.cgi/hadoop/common/hadoop-3.2.1/hadoop-3.2.1.tar.gz) at the time of writing (3.2.1).

We will be using the [Linux Mint](https://linuxmint.com/) distribution because I love its simplicity, although the process shown here should work just fine on any similar Linux distribution such as [Ubuntu](https://ubuntu.com/).

Once the archive download is complete, extract it with any tool of your choice (graphical or using the terminal) and execute it. Make sure you have a version of Java installed, such as [OpenJDK](https://openjdk.java.net/).

Here are all the three steps in the command line:

```
wget https://apache.brunneis.com/hadoop/common/hadoop-3.2.1/hadoop-3.2.1.tar.gz
tar xf hadoop-3.2.1.tar.gz
hadoop-3.2.1/bin/hadoop version
```

We will be using the two example data files that they provide in [their GitHub repository](https://github.com/tomwhite/hadoop-book/tree/master/input/ncdc/all), although the full dataset is offered by the [National Climatic Data Center](https://www.ncdc.noaa.gov/) (NCDC).

We will also unzip and concatenate both files into a single text file, to make it easier to work with. As a single command pipeline:

```
curl https://raw.githubusercontent.com/tomwhite/hadoop-book/master/input/ncdc/all/190{1,2}.gz | gunzip > 190x
```

This should create a `190x` text file in the current directory, which will be our input data.

## Processing data

To take advantage of Hadoop, we have to design our code to work in the MapReduce model. Both the map and reduce phase work on key-value pairs as input and output, and both have a programmer-defined function.

We will use Java, because it’s a dependency that we already have anyway, so might as well.

Our map function needs to extract the year and air temperature, which will prepare the data for later use (finding the maximum temperature for each year). We will also drop bad records here (if the temperature is missing, suspect or erroneous).

Copy or reproduce the following code in a file called `MaxTempMapper.java`, using any text editor of your choice:

```
import java.io.IOException;

import org.apache.hadoop.io.IntWritable;
import org.apache.hadoop.io.LongWritable;
import org.apache.hadoop.io.Text;
import org.apache.hadoop.mapreduce.Mapper;

public class MaxTempMapper extends Mapper<LongWritable, Text, Text, IntWritable> {
    private static final int TEMP_MISSING = 9999;
    private static final String GOOD_QUALITY_RE = "[01459]";

    @Override
    public void map(LongWritable key, Text value, Context context)
            throws IOException, InterruptedException {
        String line = value.toString();
        String year = line.substring(15, 19);
        String temp = line.substring(87, 92).replaceAll("^\\+", "");
        String quality = line.substring(92, 93);

        int airTemperature = Integer.parseInt(temp);
        if (airTemperature != TEMP_MISSING && quality.matches(GOOD_QUALITY_RE)) {
            context.write(new Text(year), new IntWritable(airTemperature));
        }
    }
}
```

Now, let’s create the `MaxTempReducer.java` file. Its job is to reduce the data from multiple values into just one. We do that by keeping the maximum out of all the values we receive:

```
import java.io.IOException;
import java.util.Iterator;

import org.apache.hadoop.io.IntWritable;
import org.apache.hadoop.io.Text;
import org.apache.hadoop.mapreduce.Reducer;

public class MaxTempReducer extends Reducer<Text, IntWritable, Text, IntWritable> {
    @Override
    public void reduce(Text key, Iterable<IntWritable> values, Context context)
            throws IOException, InterruptedException {
        Iterator<IntWritable> iter = values.iterator();
        if (iter.hasNext()) {
            int maxValue = iter.next().get();
            while (iter.hasNext()) {
                maxValue = Math.max(maxValue, iter.next().get());
            }
            context.write(key, new IntWritable(maxValue));
        }
    }
}
```

Except for some Java weirdness (…why can’t we just iterate over an `Iterator`? Or why can’t we just manually call `next()` on an `Iterable`?), our code is correct. There can’t be a maximum if there are no elements, and we want to avoid dummy values such as `Integer.MIN_VALUE`.

We can also take a moment to appreciate how absolutely tiny this code is, and it’s Java! Hadoop’s API is really awesome and lets us write such concise code to achieve what we need.

Last, let’s write the `main` method, or else we won’t be able to run it. In our new file `MaxTemp.java`:

```
import org.apache.hadoop.fs.Path;
import org.apache.hadoop.io.IntWritable;
import org.apache.hadoop.io.Text;
import org.apache.hadoop.mapreduce.Job;
import org.apache.hadoop.mapreduce.lib.input.FileInputFormat;
import org.apache.hadoop.mapreduce.lib.output.FileOutputFormat;

public class MaxTemp {
    public static void main(String[] args) throws Exception {
        if (args.length != 2) {
            System.err.println("usage: java MaxTemp <input path> <output path>");
            System.exit(-1);
        }

        Job job = Job.getInstance();

        job.setJobName("Max temperature");
        job.setJarByClass(MaxTemp.class);
        job.setMapperClass(MaxTempMapper.class);
        job.setReducerClass(MaxTempReducer.class);
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

At last, we can run it (also specifying the dependencies in the classpath, this one’s a mouthful):

```
java -cp ".:hadoop-3.2.1/share/hadoop/common/*:hadoop-3.2.1/share/hadoop/common/lib/*:hadoop-3.2.1/share/hadoop/mapreduce/*:hadoop-3.2.1/share/hadoop/mapreduce/lib/*:hadoop-3.2.1/share/hadoop/yarn/*:hadoop-3.2.1/share/hadoop/yarn/lib/*:hadoop-3.2.1/share/hadoop/hdfs/*:hadoop-3.2.1/share/hadoop/hdfs/lib/*" MaxTemp 190x results
```

Hooray! We should have a new `results/` folder along with the following files:

```
$ ls results
part-r-00000  _SUCCESS
$ cat results/part-r-00000
1901	317
1902	244
```

It worked! Now this example was obviously tiny, but hopefully enough to demonstrate how to get the basics running on real world data.