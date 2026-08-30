//! # html_2_json
//!
//! Parse arbitrary HTML content into a structured, serialisable JSON tree.
//!
//! ## Quick start
//!
//! ```rust
//! use html_2_json::{parse_html, ContentBlock};
//!
//! let blocks: Vec<ContentBlock> = parse_html("<p>Hello <strong>world</strong></p>");
//! let json = serde_json::to_string_pretty(&blocks).unwrap();
//! println!("{json}");
//! ```
//!
//! ## Supported HTML elements
//!
//! | Element | Output type |
//! |---------|-------------|
//! | `<p>` | [`ContentBlock::Paragraph`] |
//! | `<h1>`–`<h6>` | [`ContentBlock::Heading`] |
//! | `<img>` | [`ContentBlock::Image`] |
//! | `<figure>` + `<figcaption>` | [`ContentBlock::Figure`] |
//! | `<pre><code>` / `<div class="highlight">` | [`ContentBlock::CodeBlock`] |
//! | `<ul>` / `<ol>` (with nested lists) | [`ContentBlock::List`] |
//! | `<dl>` / `<dt>` / `<dd>` | [`ContentBlock::DefinitionList`] |
//! | `<table>` | [`ContentBlock::Table`] |
//! | `<blockquote>` | [`ContentBlock::Quote`] |
//! | `<video>` | [`ContentBlock::Video`] |
//! | `<audio>` | [`ContentBlock::Audio`] |
//! | `<iframe>` | [`ContentBlock::Embed`] |
//! | `<hr>` | [`ContentBlock::Separator`] |
//! | `<strong>` / `<b>` | [`InlineNode::Bold`] |
//! | `<em>` / `<i>` | [`InlineNode::Italic`] |
//! | `<a>` | [`InlineNode::Link`] |
//! | `<code>` (inline) | [`InlineNode::InlineCode`] |
//! | `<br>` | [`InlineNode::Break`] |

mod models;
mod parser;

// ── Public API ────────────────────────────────────────────────────────────────

/// Parse a fragment or full document of HTML into a `Vec<ContentBlock>`.
///
/// This is the single entry point for the library.
///
/// ```rust
/// use html_2_json::parse_html;
///
/// let blocks = parse_html("<h1>Hello</h1><p>World</p>");
/// assert_eq!(blocks.len(), 2);
/// ```
pub use parser::parse_html;

// All public model types re-exported at the crate root.
pub use models::{
    ContentBlock,
    DefinitionItem,
    InlineNode,
    ListItem,
    TableCell,
    TableRow,
};

pub mod ffi;
