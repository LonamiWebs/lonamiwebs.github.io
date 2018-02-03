```meta
title: Build your own PC
published: 2020-02-25T02:00:12+00:00
updated: 2020-03-18T09:38:46+00:00
```

_…where PC obviously stands for Personal Crawler_.

----------

This post contains the source code for a very simple crawler written in Java. You can compile and run it on any file or directory, and it will calculate the frequency of all the words it finds.

## Source code

Paste the following code in a new file called `Crawl.java`:

```
import java.io.*;
import java.util.*;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

class Crawl {
	// Regex used to tokenize the words from a line of text
	private final static Pattern WORDS = Pattern.compile("\\w+");

	// The file where we will cache our results
	private final static File INDEX_FILE = new File("index.bin");

	// Helper method to determine if a file is a text file or not
	private static boolean isTextFile(File file) {
		String name = file.getName().toLowerCase();
		return name.endsWith(".txt")
				|| name.endsWith(".java")
				|| name.endsWith(".c")
				|| name.endsWith(".cpp")
				|| name.endsWith(".h")
				|| name.endsWith(".hpp")
				|| name.endsWith(".html")
				|| name.endsWith(".css")
				|| name.endsWith(".js");
	}

	// Normalizes a string by converting it to lowercase and removing accents
	private static String normalize(String string) {
		return string.toLowerCase()
				.replace("á", "a")
				.replace("é", "e")
				.replace("í", "i")
				.replace("ó", "o")
				.replace("ú", "u");
	}

	// Recursively fills the map with the count of words found on all the text files
	static void fillWordMap(Map<String, Integer> map, File root) throws IOException {
		// Our file queue begins with the root
		Queue<File> fileQueue = new ArrayDeque<>();
		fileQueue.add(root);

		// For as long as the queue is not empty...
		File file;
		while ((file = fileQueue.poll()) != null) {
			if (!file.exists() || !file.canRead()) {
				// ...ignore files for which we don't have permission...
				System.err.println("warning: cannot read file: " + file);
			} else if (file.isDirectory()) {
				// ...else if it's a directory, extend our queue with its files...
				File[] files = file.listFiles();
				if (files == null) {
					System.err.println("warning: cannot list dir: " + file);
				} else {
					fileQueue.addAll(Arrays.asList(files));
				}
			} else if (isTextFile(file)) {
				// ...otherwise, count the words in the file.
				countWordsInFile(map, file);
			}
		}
	}

	// Counts the words in a single file and adds the count to the map.
	public static void countWordsInFile(Map<String, Integer> map, File file) throws IOException {
		BufferedReader reader = new BufferedReader(new FileReader(file));

		String line;
		while ((line = reader.readLine()) != null) {
			Matcher matcher = WORDS.matcher(line);
			while (matcher.find()) {
				String token = normalize(matcher.group());
				Integer count = map.get(token);
				if (count == null) {
					map.put(token, 1);
				} else {
					map.put(token, count + 1);
				}
			}
		}

		reader.close();
	}

	// Prints the map of word count to the desired output stream.
	public static void printWordMap(Map<String, Integer> map, PrintStream writer) {
		List<String> keys = new ArrayList<>(map.keySet());
		Collections.sort(keys);
		for (String key : keys) {
			writer.println(key + "\t" + map.get(key));
		}
	}

	@SuppressWarnings("unchecked")
	public static void main(String[] args) throws IOException, ClassNotFoundException {
		// Validate arguments
		if (args.length == 1 && args[0].equals("--help")) {
			System.err.println("usage: java Crawl [input]");
			return;
		}

		File root = new File(args.length > 0 ? args[0] : ".");

		// Loading or generating the map where we aggregate the data  {word: count}
		Map<String, Integer> map;
		if (INDEX_FILE.isFile()) {
			System.err.println("Found existing index file: " + INDEX_FILE);
			try (ObjectInputStream ois = new ObjectInputStream(new FileInputStream(INDEX_FILE))) {
				map = (Map<String, Integer>) ois.readObject();
			}
		} else {
			System.err.println("Index file not found: " + INDEX_FILE + "; indexing...");
			map = new TreeMap<>();
			fillWordMap(map, root);
			// Cache the results to avoid doing the work a next time
			try (ObjectOutputStream out = new ObjectOutputStream(new FileOutputStream(INDEX_FILE))) {
				out.writeObject(map);
			}
		}

		// Ask the user in a loop to query for words
		Scanner scanner = new Scanner(System.in);
		while (true) {
			System.out.print("Escriba palabra a consultar (o Enter para salir): ");
			System.out.flush();
			String line = scanner.nextLine().trim();
			if (line.isEmpty()) {
				break;
			}

			line = normalize(line);
			Integer count = map.get(line);
			if (count == null) {
				System.out.println(String.format("La palabra \"%s\" no está presente", line));
			} else if (count == 1) {
				System.out.println(String.format("La palabra \"%s\" está presente 1 vez", line));
			} else {
				System.out.println(String.format("La palabra \"%s\" está presente %d veces", line, count));
			}
		}
	}
}
```

It can be compiled and executed as follows:

```
javac Crawl.java
java Crawl
```

Instead of copy-pasting the code, you may also download it as a `.zip`:

*(contents removed)*

## Addendum

The following simple function can be used if one desires to print the contents of a file:

```
public static void printFile(File file) {
	if (isTextFile(file)) {
		System.out.println('\n' + file.getName());
		try (BufferedReader reader = new BufferedReader(new FileReader(file))) {
			String line;
			while ((line = reader.readLine()) != null) {
				System.out.println(line);
			}
		} catch (FileNotFoundException ignored) {
			System.err.println("warning: file disappeared while reading: " + file);
		} catch (IOException e) {
			e.printStackTrace();
		}
	}
}
```
