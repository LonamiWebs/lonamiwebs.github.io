Source for lonami.dev's website.

# Contributing

Typo fixes to the `content` are welcome.

Changes elsewhere will *not* be considered. This is my personal website.
The `src` folder *is* open source though, so you can make use of it as the license allows.

**Only the `src` folder has a license.**
You are allowed to cite the published `content` and copy excerpts of it to other spaces,
but please do not copy it wholesale or pretend it is your own.

# Building Site

Requires [Rust](https://www.rust-lang.org/).

Build:

```sh
pushd site; cargo build {,--release}; popd
```

Optionally symlink:

```sh
ln -s target/{debug,release}/site{,.exe}
```

See available commands by running:

```sh
./site -h
```

# File Structure

## content

The website's content.

## src

The website's `site`-executable source code.

## www

Built page.
