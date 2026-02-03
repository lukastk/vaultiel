#!/bin/bash
# Graph Export with Vaultiel CLI
# This script demonstrates exporting vault data to graph database formats.

set -e

VAULT="${VAULT:-/tmp/vaultiel-export-demo}"
OUTPUT_DIR="${OUTPUT_DIR:-/tmp/vaultiel-exports}"

echo "=== Vaultiel CLI Demo: Graph Export ==="
echo "Using vault: $VAULT"
echo "Output dir: $OUTPUT_DIR"
echo

mkdir -p "$VAULT" "$OUTPUT_DIR"

# Create a small interconnected vault
vaultiel --vault "$VAULT" create "concepts/Graph Theory.md" \
    --frontmatter '{"type": "concept", "difficulty": "intermediate"}' \
    --content "# Graph Theory

The study of graphs and networks.

## Key Concepts
- [[concepts/Nodes]]
- [[concepts/Edges]]
- [[concepts/Algorithms]]

## Applications
- Social networks
- Route planning
- [[projects/Knowledge Graph]]

#math #computer-science"

vaultiel --vault "$VAULT" create "concepts/Nodes.md" \
    --frontmatter '{"type": "concept"}' \
    --content "# Nodes

Vertices in a [[concepts/Graph Theory|graph]].

Also called vertices or points.

#math"

vaultiel --vault "$VAULT" create "concepts/Edges.md" \
    --frontmatter '{"type": "concept"}' \
    --content "# Edges

Connections between [[concepts/Nodes]] in a [[concepts/Graph Theory|graph]].

Can be directed or undirected.

#math"

vaultiel --vault "$VAULT" create "concepts/Algorithms.md" \
    --frontmatter '{"type": "concept", "difficulty": "advanced"}' \
    --content "# Graph Algorithms

Algorithms that operate on [[concepts/Graph Theory|graphs]].

## Examples
- Dijkstra's shortest path
- BFS/DFS traversal
- PageRank

#computer-science #algorithms"

vaultiel --vault "$VAULT" create "projects/Knowledge Graph.md" \
    --frontmatter '{"type": "project", "status": "active"}' \
    --content "# Knowledge Graph Project

Building a knowledge graph using [[concepts/Graph Theory]].

## Goals
- Model relationships between [[concepts/Nodes|entities]]
- Query with graph algorithms

#project"

echo "Created demo vault"
echo

# --- Basic Neo4j Cypher Export ---
echo "--- Neo4j Cypher Export (Basic) ---"
vaultiel --vault "$VAULT" export-graph --format cypher --output "$OUTPUT_DIR/basic.cypher"
echo "Exported to: $OUTPUT_DIR/basic.cypher"
echo
echo "Preview:"
head -20 "$OUTPUT_DIR/basic.cypher"
echo "..."
echo

# --- Neo4j with MERGE (idempotent) ---
echo "--- Neo4j Cypher Export (with MERGE) ---"
vaultiel --vault "$VAULT" export-graph --format cypher --use-merge --output "$OUTPUT_DIR/merge.cypher"
echo "Exported to: $OUTPUT_DIR/merge.cypher"
echo
echo "Preview (showing MERGE statements):"
grep -m 3 "MERGE" "$OUTPUT_DIR/merge.cypher" || head -10 "$OUTPUT_DIR/merge.cypher"
echo

# --- Neo4j with Tags ---
echo "--- Neo4j Cypher Export (with Tags) ---"
vaultiel --vault "$VAULT" export-graph --format cypher --include-tags --output "$OUTPUT_DIR/with-tags.cypher"
echo "Exported to: $OUTPUT_DIR/with-tags.cypher"
echo
echo "Tag nodes:"
grep -i "tag" "$OUTPUT_DIR/with-tags.cypher" | head -5 || echo "(check file for tag content)"
echo

# --- Neo4j with Frontmatter ---
echo "--- Neo4j Cypher Export (with Frontmatter) ---"
vaultiel --vault "$VAULT" export-graph --format cypher --include-frontmatter --output "$OUTPUT_DIR/with-frontmatter.cypher"
echo "Exported to: $OUTPUT_DIR/with-frontmatter.cypher"
echo
echo "Preview (showing properties):"
head -10 "$OUTPUT_DIR/with-frontmatter.cypher"
echo

# --- JSON-LD Export (Basic) ---
echo "--- JSON-LD Export (Basic) ---"
vaultiel --vault "$VAULT" export-graph --format json-ld --pretty --output "$OUTPUT_DIR/basic.jsonld"
echo "Exported to: $OUTPUT_DIR/basic.jsonld"
echo
echo "Preview:"
head -30 "$OUTPUT_DIR/basic.jsonld"
echo "..."
echo

# --- JSON-LD with Custom Base URI ---
echo "--- JSON-LD Export (with Base URI) ---"
vaultiel --vault "$VAULT" export-graph --format json-ld --pretty \
    --base-uri "https://example.com/vault/" \
    --output "$OUTPUT_DIR/with-uri.jsonld"
echo "Exported to: $OUTPUT_DIR/with-uri.jsonld"
echo
echo "Context and first item:"
head -25 "$OUTPUT_DIR/with-uri.jsonld"
echo

# --- Full Export (all options) ---
echo "--- Full Export (Tags + Headings + Frontmatter) ---"
vaultiel --vault "$VAULT" export-graph --format json-ld --pretty \
    --include-tags \
    --include-headings \
    --include-frontmatter \
    --base-uri "https://myknowledge.example.com/" \
    --output "$OUTPUT_DIR/full.jsonld"
echo "Exported to: $OUTPUT_DIR/full.jsonld"
echo
echo "File size: $(wc -c < "$OUTPUT_DIR/full.jsonld") bytes"
echo

# --- Output to stdout (for piping) ---
echo "--- Export to stdout (pipe to other tools) ---"
echo "Example: vaultiel export-graph --format cypher | cypher-shell"
echo
vaultiel --vault "$VAULT" export-graph --format json-ld 2>/dev/null | jq 'keys'
echo

# --- Summary ---
echo
echo "=== Export Summary ==="
echo "Files created in $OUTPUT_DIR:"
ls -la "$OUTPUT_DIR"
echo
echo "To load into Neo4j:"
echo "  cat $OUTPUT_DIR/merge.cypher | cypher-shell -u neo4j -p password"
echo
echo "To clean up:"
echo "  rm -rf $VAULT $OUTPUT_DIR"
