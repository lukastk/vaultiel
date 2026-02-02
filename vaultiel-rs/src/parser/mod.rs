//! Parsers for Obsidian markdown syntax.

pub mod block_id;
pub mod code_block;
pub mod frontmatter;
pub mod heading;
pub mod inline_attr;
pub mod tag;
pub mod wikilink;

pub use block_id::parse_block_ids;
pub use code_block::{find_code_block_ranges, CodeBlockRange};
pub use frontmatter::{
    extract_frontmatter, parse_frontmatter, parse_frontmatter_with_path,
    serialize_frontmatter, split_frontmatter, update_frontmatter,
};
pub use heading::{find_heading_by_slug, find_heading_by_text, parse_headings, slugify};
pub use inline_attr::parse_inline_attrs;
pub use tag::parse_tags;
pub use wikilink::{parse_all_links, parse_embeds, parse_links};
