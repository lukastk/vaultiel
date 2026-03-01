//! Structured search across vault notes.

pub mod matcher;
pub mod parser;
pub mod types;

pub use matcher::evaluate_note;
pub use parser::parse_query;
pub use types::*;
