//! An tree structure over an arbitrary node type that is cryptographically hashed as a Merkle
//! n-ary acyclic directed graph (i.e. a Merkle DAG).
//!
//! # Supporting Reference Literature
//!
//! - [Graphs in Rust: An Introduction to Petgraph][0]: A good primer for using the Petgraph Rust
//! crate, especially considering that the crate's documentation is often terse and lacking in
//! examples.
//! - [Iterative Depth First Traversal of Graph][1]: The description of an algorithm to perform a
//! depth-first traversal (*DFS*) of a tree in an iterative fashion. In general here we are
//! avoiding recursive algorithms as the inputs to these trees can be user provided so it is hard
//! to know the depth of such trees ahead of time.
//! - [Iterative Postorder Traversal of N-ary Tree][2]: The description of an algorithm to perform
//! a depth-first (*DFS*) post-order traversal of an n-ary tree (i.e. more than 2 children). This
//! is the algorithm required to compute the hashes for a Merkle tree. That is, children must be
//! hashed before the parent can be hashed.
//! - [No More Tears, No More Knots: Arena-Allocated Trees in Rust][3]: A good article explaining
//! the challenges when working with ownership and trees in Rust. Covers arena-allocated trees,
//! which is a strategy that Petgraph's default `Graph` implementation uses.
//! - [Mutable post-order iterator over tree structure][4]: A Rust users thread discussing
//! challenges while traversing a graph and mutating nodes. A key insight is to use arena
//! allocation for ownership of the graph's nodes.
//! [Idiomatic tree and graph like structures in Rust][5]: Short write-up explaining Rust
//! ownership, trees, and the common solution of using memory arenas.
//! - [Graph & Tree Traversals in Rust][6]: Covers Rust memory ownership rules, the idea of arena
//! allocators and the Visitor Pattern as strategies to perfom immutable and mutable tree
//! traversals.
//! - [Rust Unofficial Design Patterns: Visitor][7]: A visitor encapsulates an algorithm that
//! operates over a heterogeneous collection of objects.
//! - [Advanced Data Structures Part 1: Directed Acyclic Graph (DAG)][8]: Introduction to Directed
//! Acyclic Graphs (*DAG*).
//! - [Directed Acyclic Graphs (DAGs)][9]: Chapter from "Version Control by Example" which covers
//! DAGs
//! - [Wikipedia: Merkle tree][10]: Introduction to Merkle trees and their uses (hint: we're using
//! non-binary aka n-ary Merkle trees in this crate)
//! - [IPFS: What is a Merkle DAG?][11]: Short description of a Merkle DAG, that is an n-ary tree
//! and non-leaf nodes can contain data (hint: we're using Merkle DAGs in this crate)
//! - [YouTube: Data corruption and Merkle trees][12]
//! - [YouTube: Merkle Tree with real world examples][13]
//! - [YouTube: IPFS Merkle DAG][14]
//! - [YouTube: Cryptography: Merkle Tree][15]
//! - [YouTube: Blockchain Primer - Merkle Tree, DAG, Consensus Nonce][16]
//!
//! [0]:  https://depth-first.com/articles/2020/02/03/graphs-in-rust-an-introduction-to-petgraph/
//! [1]:  https://www.geeksforgeeks.org/iterative-depth-first-traversal/
//! [2]:  https://www.geeksforgeeks.org/iterative-postorder-traversal-of-n-ary-tree/
//! [3]:  https://dev.to/deciduously/no-more-tears-no-more-knots-arena-allocated-trees-in-rust-44k6
//! [4]:  https://users.rust-lang.org/t/mutable-post-order-iterator-over-tree-structure/42701
//! [5]:  https://rust-leipzig.github.io/architecture/2016/12/20/idiomatic-trees-in-rust/
//! [6]:  https://sachanganesh.com/programming/graph-tree-traversals-in-rust/
//! [7]:  https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html
//! [8]:  https://medium.com/@hamzasurti/advanced-data-structures-part-1-directed-acyclic-graph-dag-c1d1145b5e5a
//! [9]:  https://ericsink.com/vcbe/html/directed_acyclic_graphs.html
//! [10]: https://en.wikipedia.org/wiki/Merkle_tree
//! [11]: https://discuss.ipfs.tech/t/what-is-a-merkle-dag/386
//! [12]: https://youtu.be/rsx1nt2bxf8
//! [13]: https://youtu.be/qHMLy5JjbjQ
//! [14]: https://youtu.be/5XLLiGWxtdM
//! [15]: https://youtu.be/Y542oUfYdq4
//! [16]: https://youtu.be/8qqQG7da6AA

#![warn(
    clippy::unwrap_in_result,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn,
    missing_docs
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_inception,
    clippy::module_name_repetitions
)]

mod graph;
mod hash;
mod tar;

pub use crate::tar::{
    read::TarReadError,
    write::{TarWriter, TarWriterError},
};
pub use graph::{
    read_key_value_line, read_key_value_line_opt, write_key_value_line, write_key_value_line_opt,
    GraphError, HashedNode, NameStr, NodeChild, NodeKind, NodeWithChildren, ObjectTree, ReadBytes,
    WriteBytes,
};
pub use hash::{Hash, HashParseError};
