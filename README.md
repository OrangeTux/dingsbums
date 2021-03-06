# Dingsbums

Rust implementation of [knowledge storage system][zettelkasten-wiki] called
"ZettelKasten" as invented by [Niklas Luhmann][luhman].

## Installation

Clone this repo:

```
$ git clone https://github.com/OrangeTux/dingsbums.git dingsbums
$ cargo install dingsbums
```


## Usage

When used for the first time a new `Dingsbums` must be created:

```
$ ztl init
```

Now you can add a root `Zettel`:

```
$ ztl new --no-parent
```

Or a child `Zettel`:

```
$ ztl new
```

## Data structure
The relations between zettels are stored in an Directed Cyclic graph.
A directed graph allows see how ideas and knowledge develops.
As a zettel can be linked to other zettels it might be possible to form a cycle.

## Debugging

The log level of `Dingsbums` can be modified by configuring the [environment
variable `RUST_LOG`][RUST_LOG]:

``` bash
$ export RUST_LOG=ztl=debug
```

## FAQ

Q1) **Why the name "Dingsbums"?**

A1) ["Dingsbums"][dingsbums] is a German word often used by a person who wants
to describe something but forgot the name of the subject. It loosely translates
to "thingy" in English.

With this project you can document your knowledge in order to recall it at a
later point in time. So you don't need to use the word "thingy" or "dingsbums"
anymore.

[dingsbums]: https://en.wiktionary.org/wiki/Dingsbums
[luhman]: https://en.wikipedia.org/wiki/Niklas_Luhmann
[zettelkasten-wiki]: https://en.wikipedia.org/wiki/Zettelkasten
[RUST_LOG]: https://docs.rs/tracing-subscriber/0.2.16/tracing_subscriber/fmt/index.html#filtering-events-with-environment-variables
