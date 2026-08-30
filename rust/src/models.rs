use serde::Serialize;

/// A top-level content node that represents a single structural element of an HTML document.
///
/// Each variant maps directly to one or more HTML tags. The entire tree can be
/// serialised to JSON via [`serde_json`] because every type derives [`Serialize`].
#[derive(Debug, Clone, Serialize)]
pub enum ContentBlock {
    /// A `<p>` element or a run of bare inline content at the block level.
    Paragraph {
        children: Vec<InlineNode>,
    },

    /// A `<h1>`–`<h6>` heading element. `level` is 1–6.
    Heading {
        level: u8,
        children: Vec<InlineNode>,
    },

    /// A standalone `<img>` element (not inside a `<figure>`).
    Image {
        url: String,
        alt: Option<String>,
    },

    /// A `<figure>` element that wraps an image with an optional `<figcaption>`.
    Figure {
        url: String,
        alt: Option<String>,
        caption: Option<String>,
    },

    /// A fenced code block — either `<pre><code>` or a `<div class="highlight">` wrapper.
    CodeBlock {
        /// Detected programming language (e.g. `"typescript"`, `"bash"`), if any.
        language: Option<String>,
        /// Raw source code text with all HTML markup stripped.
        code: String,
    },

    /// An ordered (`<ol>`) or unordered (`<ul>`) list.
    /// Each [`ListItem`] may itself contain nested sub-lists.
    List {
        ordered: bool,
        items: Vec<ListItem>,
    },

    /// A `<table>` element with one or more rows.
    Table {
        rows: Vec<TableRow>,
    },

    /// A `<blockquote>` element.
    ///
    /// Children are full [`ContentBlock`]s so `<p>` tags inside a blockquote are
    /// faithfully represented rather than flattened to inline text.
    Quote {
        children: Vec<ContentBlock>,
    },

    /// A `<dl>` definition list — a sequence of term / definition pairs.
    DefinitionList {
        items: Vec<DefinitionItem>,
    },

    /// A `<video>` element.
    Video {
        src: String,
        /// Optional thumbnail image URL (`poster` attribute).
        poster: Option<String>,
    },

    /// An `<audio>` element.
    Audio {
        src: String,
    },

    /// An `<iframe>` embed (YouTube, CodePen, etc.).
    Embed {
        src: String,
        title: Option<String>,
    },

    /// A `<hr>` horizontal rule / thematic break.
    Separator {},
}

/// An inline-level content node that lives inside a block element's `children` list.
#[derive(Debug, Clone, Serialize)]
pub enum InlineNode {
    /// Plain text content.
    Text {
        text: String,
    },

    /// `<strong>` or `<b>` — bold text with recursive inline children.
    Bold {
        children: Vec<InlineNode>,
    },

    /// `<em>` or `<i>` — italic text with recursive inline children.
    Italic {
        children: Vec<InlineNode>,
    },

    /// `<a href="…">` hyperlink.
    Link {
        url: String,
        children: Vec<InlineNode>,
    },

    /// Inline `<code>` snippet (not a fenced block).
    InlineCode {
        text: String,
    },

    /// `<br>` line break.
    Break {},
}

/// A single `<li>` item inside a [`ContentBlock::List`].
///
/// `children` holds the inline text of the item.
/// `nested` holds any nested `<ul>` or `<ol>` found **inside** the same `<li>`.
#[derive(Debug, Clone, Serialize)]
pub enum ListItem {
    Item {
        children: Vec<InlineNode>,
        /// Nested sub-lists (each is a [`ContentBlock::List`]).
        nested: Vec<ContentBlock>,
    },
}

/// A `<dt>` / `<dd>` pair from a [`ContentBlock::DefinitionList`].
#[derive(Debug, Clone, Serialize)]
pub struct DefinitionItem {
    /// The term (`<dt>`).
    pub term: Vec<InlineNode>,
    /// The definition (`<dd>`).
    pub definition: Vec<InlineNode>,
}

/// A single row inside a [`ContentBlock::Table`].
#[derive(Debug, Clone, Serialize)]
pub enum TableRow {
    Row {
        cells: Vec<TableCell>,
    },
}

/// A single cell (`<td>` or `<th>`) inside a [`TableRow`].
#[derive(Debug, Clone, Serialize)]
pub enum TableCell {
    Cell {
        children: Vec<InlineNode>,
    },
}
