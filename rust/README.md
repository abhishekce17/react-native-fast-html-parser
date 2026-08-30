# html_2_json

A high-performance, robust, and zero-patch HTML-to-structured-JSON parser library written in Rust.

`html_2_json` parses arbitrary, nested, or malformed HTML fragments into a structured AST (Abstract Syntax Tree) representing logical block elements and recursive inline formatting. The AST derives `serde::Serialize` out-of-the-box, allowing simple, clean, and direct JSON outputs.

---

## ⚡ Key Features

- **Nested Lists & Blockquotes**: Faithfully retains deep structural hierarchies (e.g. lists inside lists, paragraphs inside blockquotes).
- **$O(\log N)$ Language Detection**: Supports automatic syntax highlighting language matching for **93 different programming and configuration languages** using sorted binary search.
- **Media & Embeds**: Parsers for `<video>`, `<audio>`, and `<iframe>` components.
- **Highly Optimized**: Features allocation-free recursive inline node traversal (nodes are collected directly into pre-allocated memory buffers to eliminate thousands of small vector heap allocations).
- **Clean API**: Re-exports all AST nodes and parsing entry points flat at the crate root.

---

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
html_2_json = { path = "path/to/html_2_json" }
serde_json = "1.0"
```

---

## 🚀 Quick Start

```rust
use html_2_json::{parse_html, ContentBlock};

fn main() {
    let html = r#"
        <h1>My Article</h1>
        <p>This is a paragraph with <strong>bold</strong> text.</p>
        <pre class="language-rust"><code>fn main() {}</code></pre>
    "#;

    // Parse into a Vec of ContentBlocks
    let blocks: Vec<ContentBlock> = parse_html(html);

    // Serialize directly to JSON
    let json = serde_json::to_string_pretty(&blocks).unwrap();
    println!("{}", json);
}
```

### Output JSON
```json
[
  {
    "Heading": {
      "level": 1,
      "children": [
        {
          "Text": {
            "text": "My Article"
          }
        }
      ]
    }
  },
  {
    "Paragraph": {
      "children": [
        {
          "Text": {
            "text": "This is a paragraph with "
          }
        },
        {
          "Bold": {
            "children": [
              {
                "Text": {
                  "text": "bold"
                }
              }
            ]
          }
        },
        {
          "Text": {
            "text": " text."
          }
        }
      ]
    }
  },
  {
    "CodeBlock": {
      "language": "rust",
      "code": "fn main() {}"
    }
  }
]
```

---

## 🗺️ HTML Element Mapping

| HTML Element | Output AST Node Type |
|---|---|
| `<p>` | `ContentBlock::Paragraph` |
| `<h1>`–`<h6>` | `ContentBlock::Heading` (with level 1–6) |
| `<img>` | `ContentBlock::Image` |
| `<figure>` + `<figcaption>` | `ContentBlock::Figure` |
| `<pre><code>` / `<div class="highlight">` | `ContentBlock::CodeBlock` |
| `<ul>` / `<ol>` | `ContentBlock::List` (with nested support) |
| `<dl>` / `<dt>` / `<dd>` | `ContentBlock::DefinitionList` |
| `<table>` | `ContentBlock::Table` |
| `<blockquote>` | `ContentBlock::Quote` |
| `<video>` | `ContentBlock::Video` |
| `<audio>` | `ContentBlock::Audio` |
| `<iframe>` | `ContentBlock::Embed` |
| `<hr>` | `ContentBlock::Separator` |
| `<strong>` / `<b>` | `InlineNode::Bold` |
| `<em>` / `<i>` | `InlineNode::Italic` |
| `<a>` | `InlineNode::Link` |
| `<code>` (inline) | `InlineNode::InlineCode` |
| `<br>` | `InlineNode::Break` |

---

## 🛠️ AST Structure Reference

### Block Node (`ContentBlock`)
Represents top-level structural components:

```rust
pub enum ContentBlock {
    Paragraph { children: Vec<InlineNode> },
    Heading { level: u8, children: Vec<InlineNode> },
    Image { url: String, alt: Option<String> },
    Figure { url: String, alt: Option<String>, caption: Option<String> },
    CodeBlock { language: Option<String>, code: String },
    List { ordered: bool, items: Vec<ListItem> },
    Table { rows: Vec<TableRow> },
    Quote { children: Vec<ContentBlock> },
    DefinitionList { items: Vec<DefinitionItem> },
    Video { src: String, poster: Option<String> },
    Audio { src: String },
    Embed { src: String, title: Option<String> },
    Separator {},
}
```

### Inline Node (`InlineNode`)
Represents formatted text segments inside a block element:

```rust
pub enum InlineNode {
    Text { text: String },
    Bold { children: Vec<InlineNode> },
    Italic { children: Vec<InlineNode> },
    Link { url: String, children: Vec<InlineNode> },
    InlineCode { text: String },
    Break {},
}
```

---

To prevent memory fragmentation and lookup latency during recursive traversals:
- **Allocation-Free Inline Parsing**: `parse_inline_node` takes a mutable reference buffer `&mut Vec<InlineNode>`. Rather than allocating vectors on each nested tag recursion, child nodes are written directly into the parent vector stream.
- **Perfect Hashing Language Lookup**: Uses a compile-time static perfect hash set (`phf::Set`) supporting 93 different languages. Lookups are executed in constant $O(1)$ time with zero lookup cost.
