
# Vec-collections &emsp; [![Build Status]][travis] [![Latest Version]][crates.io] [![Docs Badge]][docs.rs]

[Build Status]: https://api.travis-ci.org/rklaehn/vec-collections.svg?branch=master
[travis]: https://travis-ci.org/rklaehn/vec-collections
[Latest Version]: https://img.shields.io/crates/v/vec-collections.svg
[crates.io]: https://crates.io/crates/vec-collections
[Docs Badge]: https://img.shields.io/badge/docs-docs.rs-green
[docs.rs]: https://docs.rs/vec-collections

# About

This is a port of [array based collections](https://github.com/rklaehn/abc) from Scala to Rust. Here is a [blog post](http://rklaehn.github.io/2015/12/18/array-based-immutable-collections/) from ages ago explaining the motivation.

A straight port would have been pretty easy, but I have tried to make the port more rusty by offering in-place operations that do not allocate.

The core algorithm that is used for all operations is a minimum comparison merge algorithm that requires fast random access
to the elements of a collection, which you of course have in case of a slice or a vec. The minimum comparison merge algorithm
will be useful as soon as the cost of a comparison is large compared to the cost of a copy, but you will still get very good
performance in the case where the comparision is roughly the same cost as a copy.

This is also a bit of a nursery for things I am currently working on.
