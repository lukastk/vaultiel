"""Type stubs for vaultiel Python bindings."""

from typing import Optional

__version__: str

class Link:
    """A link found in a note."""
    target: str
    alias: Optional[str]
    heading: Optional[str]
    block_id: Optional[str]
    embed: bool
    line: int

class Tag:
    """A tag found in a note."""
    name: str
    line: int

class Heading:
    """A heading found in a note."""
    text: str
    level: int
    line: int
    slug: str

class BlockId:
    """A block ID found in a note."""
    id: str
    line: int
    block_type: str

class Task:
    """A task found in a note."""
    file: str
    line: int
    raw: str
    symbol: str
    description: str
    indent: int
    scheduled: Optional[str]
    due: Optional[str]
    done: Optional[str]
    priority: Optional[str]
    tags: list[str]
    block_id: Optional[str]

class VaultielMetadata:
    """Vaultiel metadata for a note."""
    id: str
    created: str

class LinkRef:
    """A reference to a link (incoming or outgoing)."""
    from_note: str
    line: int
    context: str
    alias: Optional[str]
    heading: Optional[str]
    block_id: Optional[str]
    embed: bool

class Vault:
    """A Vaultiel vault instance.

    Args:
        path: Path to the vault directory.

    Example:
        >>> vault = Vault("/path/to/vault")
        >>> notes = vault.list_notes()
        >>> content = vault.get_content("my-note.md")
    """

    def __init__(self, path: str) -> None: ...

    @property
    def root(self) -> str:
        """Get the vault root path."""
        ...

    def list_notes(self) -> list[str]:
        """List all notes in the vault.

        Returns:
            List of note paths relative to the vault root.
        """
        ...

    def list_notes_matching(self, pattern: str) -> list[str]:
        """List notes matching a glob pattern.

        Args:
            pattern: Glob pattern to match (e.g., "daily/*.md").

        Returns:
            List of matching note paths.
        """
        ...

    def note_exists(self, path: str) -> bool:
        """Check if a note exists.

        Args:
            path: Path to the note (with or without .md extension).

        Returns:
            True if the note exists, False otherwise.
        """
        ...

    def get_content(self, path: str) -> str:
        """Get the full content of a note (including frontmatter).

        Args:
            path: Path to the note.

        Returns:
            The note's full content as a string.
        """
        ...

    def get_body(self, path: str) -> str:
        """Get the body of a note (content without frontmatter).

        Args:
            path: Path to the note.

        Returns:
            The note's body content as a string.
        """
        ...

    def get_frontmatter(self, path: str) -> Optional[str]:
        """Get the frontmatter of a note as a JSON string.

        Args:
            path: Path to the note.

        Returns:
            JSON string of the frontmatter, or None if no frontmatter.
        """
        ...

    def get_frontmatter_dict(self, path: str) -> Optional[dict]:
        """Get the frontmatter of a note as a Python dict.

        Args:
            path: Path to the note.

        Returns:
            Dict of frontmatter key-value pairs, or None if no frontmatter.
        """
        ...

    def create_note(self, path: str, content: str) -> None:
        """Create a new note.

        Args:
            path: Path for the new note.
            content: Content of the note.

        Raises:
            RuntimeError: If the note cannot be created.
        """
        ...

    def delete_note(self, path: str) -> None:
        """Delete a note.

        Args:
            path: Path to the note to delete.

        Raises:
            RuntimeError: If the note cannot be deleted.
        """
        ...

    def rename_note(self, from_path: str, to_path: str) -> None:
        """Rename a note (without link propagation).

        Args:
            from_path: Current path of the note.
            to_path: New path for the note.

        Raises:
            RuntimeError: If the note cannot be renamed.
        """
        ...

    def resolve_note(self, query: str) -> str:
        """Resolve a note name or alias to a path.

        Args:
            query: Note name, alias, or partial path.

        Returns:
            The resolved path to the note.

        Raises:
            RuntimeError: If the note cannot be resolved.
        """
        ...

    def get_links(self, path: str) -> list[Link]:
        """Get all links from a note.

        Args:
            path: Path to the note.

        Returns:
            List of Link objects found in the note.
        """
        ...

    def get_tags(self, path: str) -> list[Tag]:
        """Get all tags from a note.

        Args:
            path: Path to the note.

        Returns:
            List of Tag objects found in the note.
        """
        ...

    def get_headings(self, path: str) -> list[Heading]:
        """Get all headings from a note.

        Args:
            path: Path to the note.

        Returns:
            List of Heading objects found in the note.
        """
        ...

    def get_block_ids(self, path: str) -> list[BlockId]:
        """Get all block IDs from a note.

        Args:
            path: Path to the note.

        Returns:
            List of BlockId objects found in the note.
        """
        ...

    def get_tasks(self, path: str) -> list[Task]:
        """Get all tasks from a note.

        Args:
            path: Path to the note.

        Returns:
            List of Task objects found in the note.
        """
        ...

    def get_incoming_links(self, path: str) -> list[LinkRef]:
        """Get incoming links to a note.

        Args:
            path: Path to the note.

        Returns:
            List of LinkRef objects representing links from other notes.
        """
        ...

    def get_outgoing_links(self, path: str) -> list[LinkRef]:
        """Get outgoing links from a note.

        Args:
            path: Path to the note.

        Returns:
            List of LinkRef objects representing links to other notes.
        """
        ...

    def init_metadata(self, path: str, force: bool = False) -> Optional[VaultielMetadata]:
        """Initialize vaultiel metadata for a note.

        Adds a vaultiel field with a UUID and creation timestamp to the note's
        frontmatter if it doesn't already exist.

        Args:
            path: Path to the note.
            force: If True, overwrite existing metadata.

        Returns:
            VaultielMetadata if metadata was added, None if already exists.
        """
        ...

    def get_vaultiel_metadata(self, path: str) -> Optional[VaultielMetadata]:
        """Get vaultiel metadata from a note.

        Args:
            path: Path to the note.

        Returns:
            VaultielMetadata if present, None otherwise.
        """
        ...

    def find_by_id(self, id: str) -> Optional[str]:
        """Find a note by its vaultiel ID.

        Args:
            id: The UUID to search for.

        Returns:
            Path to the note if found, None otherwise.
        """
        ...

def parse_links(content: str) -> list[Link]:
    """Parse links from markdown content.

    Args:
        content: Markdown content to parse.

    Returns:
        List of Link objects found in the content.
    """
    ...

def parse_content_tags(content: str) -> list[Tag]:
    """Parse tags from markdown content.

    Args:
        content: Markdown content to parse.

    Returns:
        List of Tag objects found in the content.
    """
    ...

def parse_content_headings(content: str) -> list[Heading]:
    """Parse headings from markdown content.

    Args:
        content: Markdown content to parse.

    Returns:
        List of Heading objects found in the content.
    """
    ...

def parse_content_block_ids(content: str) -> list[BlockId]:
    """Parse block IDs from markdown content.

    Args:
        content: Markdown content to parse.

    Returns:
        List of BlockId objects found in the content.
    """
    ...
