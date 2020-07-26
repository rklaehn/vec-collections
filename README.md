
# Vec-collections &emsp; [![Build Status]][travis] [![Latest Version]][crates.io] [![Docs Badge]][docs.rs]

[Build Status]: https://api.travis-ci.org/rklaehn/vec-collections.svg?branch=master
[travis]: https://travis-ci.org/rklaehn/vec-collections
[Latest Version]: https://img.shields.io/crates/v/vec-collections.svg
[crates.io]: https://crates.io/crates/vec-collections
[Docs Badge]: https://img.shields.io/badge/docs-docs.rs-green
[docs.rs]: https://docs.rs/vec-collections

# About

Order based collections that are backed by smallvec.

# When to use this crate

The collections in this crate are [succinct data structures], having only a small constant overhead over their contents. They will therefore be a good choice if the data size in memory is the most important concern.

They will also be very efficient for collections with a small number of elements. E.g. if you have a set of u32 that you know will rarely exceed 4 elements, you can use a VecSet backed by a SmallVec<[u32; 4]> and have no allocations at all unless you exceed 4 elements.

They will have superior performance to the standard ordered collection for bulk creation using [collect] and for lookup. For cases with a fast [Ord] implementation (e.g. primitive types, random byte arrays), creation and lookup performance will even be superior to the standard hash based collections.

The downside is that update operations (e.g. insert, removal) have O(N) performance. So if you have a case where you have a collection that is large and is frequently updated after creation, performance will be *very* bad. You have been warned.

# Creation

[VecSet] and [VecMap] support FromIterator for creation using collect

# Operations

# Benchmarks

There are some very basic benchmarks in this crate, which can be run using `cargo bench`.



# History

This is a port of [array based collections] from Scala to Rust. Here is a [blog post](http://rklaehn.github.io/2015/12/18/array-based-immutable-collections/) from ages ago explaining the motivation.

A straight port would have been pretty easy, but I have tried to make the port more rusty by offering in-place operations that do not allocate.

The core algorithm that is used for all operations is a minimum comparison merge algorithm that requires fast random access
to the elements of a collection, which you of course have in case of a slice or a vec. The minimum comparison merge algorithm
will be useful as soon as the cost of a comparison is large compared to the cost of a copy, but you will still get very good
performance in the case where the comparision is roughly the same cost as a copy.

This is also a bit of a nursery for things I am currently working on.

[collect]: https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect
[succinct data structures]: https://en.wikipedia.org/wiki/Succinct_data_structure
[Ord]: https://doc.rust-lang.org/std/cmp/trait.Ord.html
[array based collections]: https://github.com/rklaehn/abc
