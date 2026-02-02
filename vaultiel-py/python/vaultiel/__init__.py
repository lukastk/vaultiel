"""
Vaultiel - A library for programmatically interacting with Obsidian-style vaults.

This module provides Python bindings to the Vaultiel Rust library, enabling
fast and efficient operations on Obsidian-compatible markdown vaults.

Example:
    >>> from vaultiel import Vault
    >>> vault = Vault("/path/to/vault")
    >>> notes = vault.list_notes()
    >>> content = vault.get_content("my-note.md")
    >>> links = vault.get_links("my-note.md")
"""

from vaultiel.vaultiel_py import (
    Vault,
    Link,
    Tag,
    Heading,
    BlockId,
    Task,
    VaultielMetadata,
    LinkRef,
    parse_links,
    parse_content_tags,
    parse_content_headings,
    parse_content_block_ids,
)

__all__ = [
    "Vault",
    "Link",
    "Tag",
    "Heading",
    "BlockId",
    "Task",
    "VaultielMetadata",
    "LinkRef",
    "parse_links",
    "parse_content_tags",
    "parse_content_headings",
    "parse_content_block_ids",
]

__version__ = "0.1.0"
