//! Link graph and relationship tracking.

mod link_graph;
pub mod resolution;

pub use link_graph::{IncomingLink, LinkGraph, LinkInfo};
pub use resolution::resolve_link_target;
