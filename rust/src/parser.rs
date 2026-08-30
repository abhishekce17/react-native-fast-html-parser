use crate::models::{
    ContentBlock, DefinitionItem, InlineNode, ListItem, TableCell, TableRow,
};
use ego_tree::NodeRef;
use scraper::{Html, Node, Selector};

// ─────────────────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────────────────

/// Parse a fragment or full document of HTML into a `Vec<ContentBlock>`.
///
/// Handles both `Html::parse_fragment` output (which wraps content in a virtual
/// `<html><body>`) and raw root nodes as a fallback.
///
/// # Example
/// ```rust
/// use html_2_json::parse_html;
///
/// let blocks = parse_html("<h1>Title</h1><p>Body</p>");
/// assert_eq!(blocks.len(), 2);
/// ```
pub fn parse_html(html: &str) -> Vec<ContentBlock> {
    let html_dom = Html::parse_fragment(html);
    let body_selector = Selector::parse("body").unwrap();

    if let Some(body) = html_dom.select(&body_selector).next() {
        parse_block_nodes(body.children())
    } else {
        let root = html_dom.tree.root();
        if let Some(html_node) = root.first_child() {
            parse_block_nodes(html_node.children())
        } else {
            parse_block_nodes(root.children())
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Block-element classification
// ─────────────────────────────────────────────────────────────────────────────

/// Returns `true` for any tag that should be treated as a block-level element.
///
/// `tr`, `td`, `th`, `li`, `dt`, `dd` are listed here so they are **not**
/// mistaken for inline content when encountered outside their parent context.
/// They are consumed by their parent arm (`table` → tr/td/th, `ul`/`ol` → li,
/// `dl` → dt/dd).
fn is_block_element(name: &str) -> bool {
    matches!(
        name,
        "p" | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "img"
            | "figure"
            | "ul"
            | "ol"
            | "li"
            | "dl"
            | "dt"
            | "dd"
            | "table"
            | "tr"
            | "td"
            | "th"
            | "blockquote"
            | "hr"
            | "pre"
            | "div"
            | "video"
            | "audio"
            | "iframe"
            | "section"
            | "article"
            | "header"
            | "footer"
            | "aside"
            | "nav"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: recursively collect all <tr> nodes under a table
// ─────────────────────────────────────────────────────────────────────────────

fn find_trs<'a>(node: NodeRef<'a, Node>, trs: &mut Vec<NodeRef<'a, Node>>) {
    for child in node.children() {
        if let Node::Element(el) = child.value() {
            match el.name() {
                "tr" => trs.push(child),
                "thead" | "tbody" | "tfoot" => find_trs(child, trs),
                _ => {}
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: attribute sanitisation
// ─────────────────────────────────────────────────────────────────────────────

/// Strips leading/trailing backslashes and quotes from an HTML attribute value.
/// Needed because scraper can return attribute values with backslash-escaped
/// quotes when the source HTML itself contains them.
fn clean_attribute(val: &str) -> String {
    val.trim_matches(|c| c == '\\' || c == '"' || c == '\'')
        .to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: language detection for code blocks
// ─────────────────────────────────────────────────────────────────────────────

/// Detect a programming language from a `class` attribute string.
/// Checks (in order):
/// 1. `language-*` prefix
/// 2. `lang-*` prefix
/// 3. Any class token that is a known language name
fn extract_language(class_str: &str) -> Option<String> {
    let cleaned = class_str.trim_matches(|c| c == '\\' || c == '"' || c == '\'');
    for part in cleaned.split_whitespace() {
        let p = part.trim_matches(|c| c == '\\' || c == '"' || c == '\'');
        if let Some(lang) = p.strip_prefix("language-") {
            return Some(lang.to_string());
        }
        if let Some(lang) = p.strip_prefix("lang-") {
            return Some(lang.to_string());
        }
        if KNOWN_LANGUAGES.contains(p) {
            return Some(p.to_string());
        }
    }
    None
}

/// Comprehensive list of well-known programming / markup language identifiers.
static KNOWN_LANGUAGES: phf::Set<&'static str> = phf::phf_set! {
    "ada", "apache", "asm", "assembly", "awk", "bash", "c", "c#", "c++", "clojure", "cmake",
    "cobol", "coffeescript", "cpp", "csharp", "css", "d", "dart", "diff", "dockerfile", "elisp",
    "elixir", "elm", "erlang", "fish", "fortran", "fsharp", "go", "graphql", "groovy", "haskell",
    "html", "ini", "java", "javascript", "json", "jsx", "julia", "kotlin", "latex", "less", "lisp",
    "log", "lua", "makefile", "markdown", "nginx", "nim", "nix", "objc", "objective-c", "objectivec",
    "ocaml", "pascal", "perl", "php", "plaintext", "plsql", "powershell", "prql", "purescript",
    "python", "r", "racket", "ruby", "rust", "sass", "scala", "scheme", "scss", "sh", "solidity",
    "sql", "stylus", "svelte", "swift", "tcl", "terraform", "tex", "text", "tf", "toml", "tsx",
    "typescript", "vala", "vim", "vue", "wasm", "webassembly", "xml", "yaml", "zig", "zsh"
};

/// Try every available signal on the element to identify the programming language:
/// 1. `class` attribute with `language-*` or `lang-*` prefix (standard).
/// 2. Attribute *names* checked against `KNOWN_LANGUAGES` (for malformed HTML).
/// 3. Attribute *values* checked against `KNOWN_LANGUAGES`.
fn extract_language_from_element(element: &scraper::node::Element) -> Option<String> {
    // 1. Standard class-based detection
    if let Some(class_str) = element.attr("class") {
        if let Some(lang) = extract_language(class_str) {
            return Some(lang);
        }
    }
    // 2. Attribute name as language (some renderers emit `<pre typescript>`)
    for (name, _) in element.attrs() {
        let cleaned = name.trim_matches(|c| c == '\\' || c == '"' || c == '\'');
        if KNOWN_LANGUAGES.contains(&cleaned) {
            return Some(cleaned.to_string());
        }
    }
    // 3. Attribute value as language
    for (_, value) in element.attrs() {
        let cleaned = value.trim_matches(|c| c == '\\' || c == '"' || c == '\'');
        if KNOWN_LANGUAGES.contains(&cleaned) {
            return Some(cleaned.to_string());
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: collect all text content from a subtree
// ─────────────────────────────────────────────────────────────────────────────

fn collect_text(node: NodeRef<Node>) -> String {
    let mut buf = String::new();
    collect_text_recursive(node, &mut buf);
    buf
}

fn collect_text_recursive(node: NodeRef<Node>, acc: &mut String) {
    for child in node.children() {
        match child.value() {
            Node::Text(t) => acc.push_str(&t),
            Node::Element(el) => {
                if el.name() == "br" {
                    acc.push('\n');
                } else {
                    // Skip non-content noise inside code blocks
                    if matches!(el.name(), "script" | "style" | "svg") {
                        continue;
                    }
                    collect_text_recursive(child, acc);
                }
            }
            _ => {}
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Block parser
// ─────────────────────────────────────────────────────────────────────────────

/// Iterates over a sequence of sibling nodes and maps them to [`ContentBlock`]s.
///
/// Consecutive inline-level siblings are automatically accumulated and flushed
/// as a [`ContentBlock::Paragraph`] when the next block-level sibling (or end
/// of input) is encountered.
fn parse_block_nodes<'a, I>(nodes: I) -> Vec<ContentBlock>
where
    I: IntoIterator<Item = NodeRef<'a, Node>>,
{
    let mut blocks: Vec<ContentBlock> = Vec::new();
    let mut pending_inlines: Vec<InlineNode> = Vec::new();

    // Flush any accumulated inline nodes as a Paragraph (skips whitespace-only).
    let flush = |pending: &mut Vec<InlineNode>, blocks: &mut Vec<ContentBlock>| {
        if pending.is_empty() {
            return;
        }
        normalize_inline_nodes(pending);
        if !pending.is_empty() {
            blocks.push(ContentBlock::Paragraph {
                children: pending.clone(),
            });
        }
        pending.clear();
    };

    for node in nodes {
        match node.value() {
            // ── Bare text nodes ───────────────────────────────────────────────
            Node::Text(t) => {
                let text = t.to_string();
                if text.trim().is_empty() && pending_inlines.is_empty() {
                    continue; // skip inter-block whitespace
                }
                pending_inlines.push(InlineNode::Text { text });
            }

            // ── Element nodes ─────────────────────────────────────────────────
            Node::Element(element) => {
                let tag = element.name();

                if is_block_element(tag) {
                    flush(&mut pending_inlines, &mut blocks);

                    match tag {
                        // ── Paragraph ─────────────────────────────────────────
                        "p" => {
                            let mut children = parse_inline_nodes(node.children());
                            normalize_inline_nodes(&mut children);
                            blocks.push(ContentBlock::Paragraph { children });
                        }

                        // ── Headings ──────────────────────────────────────────
                        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                            let level = tag[1..].parse::<u8>().unwrap_or(1);
                            let mut children = parse_inline_nodes(node.children());
                            normalize_inline_nodes(&mut children);
                            blocks.push(ContentBlock::Heading { level, children });
                        }

                        // ── Standalone image ──────────────────────────────────
                        "img" => {
                            let url = element
                                .attr("src")
                                .map(|s| clean_attribute(&s))
                                .unwrap_or_default();
                            let alt = element.attr("alt").map(|s| clean_attribute(&s));
                            blocks.push(ContentBlock::Image { url, alt });
                        }

                        // ── Figure (image + optional caption) ─────────────────
                        "figure" => {
                            let mut url = String::new();
                            let mut alt: Option<String> = None;
                            let mut caption: Option<String> = None;

                            for child in node.children() {
                                if let Node::Element(el) = child.value() {
                                    match el.name() {
                                        "img" => {
                                            url = el
                                                .attr("src")
                                                .map(|s| clean_attribute(&s))
                                                .unwrap_or_default();
                                            alt = el.attr("alt").map(|s| clean_attribute(&s));
                                        }
                                        "figcaption" => {
                                            let text = collect_text(child).trim().to_string();
                                            if !text.is_empty() {
                                                caption = Some(text);
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            blocks.push(ContentBlock::Figure { url, alt, caption });
                        }

                        // ── Lists — supports nested lists inside <li> ──────────
                        "ul" | "ol" => {
                            let ordered = tag == "ol";
                            let items = parse_list_items(node);
                            blocks.push(ContentBlock::List { ordered, items });
                        }

                        // ── Definition list ───────────────────────────────────
                        "dl" => {
                            let items = parse_definition_list(node);
                            blocks.push(ContentBlock::DefinitionList { items });
                        }

                        // ── Table ─────────────────────────────────────────────
                        "table" => {
                            let mut tr_nodes = Vec::new();
                            find_trs(node, &mut tr_nodes);
                            let rows = tr_nodes
                                .into_iter()
                                .map(|tr| {
                                    let cells = tr
                                        .children()
                                        .filter_map(|cell| {
                                            if let Node::Element(el) = cell.value() {
                                                if matches!(el.name(), "td" | "th") {
                                                    let mut children =
                                                        parse_inline_nodes(cell.children());
                                                    normalize_inline_nodes(&mut children);
                                                    return Some(TableCell::Cell { children });
                                                }
                                            }
                                            None
                                        })
                                        .collect();
                                    TableRow::Row { cells }
                                })
                                .collect();
                            blocks.push(ContentBlock::Table { rows });
                        }

                        // ── Blockquote — full block children ──────────────────
                        "blockquote" => {
                            let children = parse_block_nodes(node.children());
                            blocks.push(ContentBlock::Quote { children });
                        }

                        // ── Divider ───────────────────────────────────────────
                        "hr" => {
                            blocks.push(ContentBlock::Separator {});
                        }

                        // ── Bare <pre><code> block ────────────────────────────
                        "pre" => {
                            // Find the inner <code> node for language detection
                            let code_child = node.children().find(|c| {
                                matches!(c.value(), Node::Element(el) if el.name() == "code")
                            });

                            let language = extract_language_from_element(element).or_else(|| {
                                code_child.and_then(|cn| {
                                    if let Node::Element(el) = cn.value() {
                                        extract_language_from_element(el)
                                    } else {
                                        None
                                    }
                                })
                            });

                            let code = code_child
                                .map(collect_text)
                                .unwrap_or_else(|| collect_text(node));

                            blocks.push(ContentBlock::CodeBlock { language, code });
                        }

                        // ── <div class="highlight"> code block or div-based tables ────
                        "div" => {
                            let classes = element.attr("class").unwrap_or("");
                            let is_highlight = classes.split_whitespace().any(|c| {
                                c.trim_matches(|ch| ch == '\\' || ch == '"' || ch == '\'')
                                    == "highlight"
                            });

                            if is_highlight {
                                if let Some(pre) = node.children().find(|c| {
                                    matches!(c.value(), Node::Element(el) if el.name() == "pre")
                                }) {
                                    let code_child = pre.children().find(|c| {
                                        matches!(c.value(), Node::Element(el) if el.name() == "code")
                                    });

                                    let language = if let Node::Element(pre_el) = pre.value() {
                                        extract_language_from_element(pre_el)
                                    } else {
                                        None
                                    }
                                    .or_else(|| extract_language_from_element(element))
                                    .or_else(|| {
                                        code_child.and_then(|cn| {
                                            if let Node::Element(el) = cn.value() {
                                                extract_language_from_element(el)
                                            } else {
                                                None
                                            }
                                        })
                                    });

                                    let code = code_child
                                        .map(collect_text)
                                        .unwrap_or_else(|| collect_text(pre));

                                    blocks.push(ContentBlock::CodeBlock { language, code });
                                } else {
                                    blocks.extend(parse_block_nodes(node.children()));
                                }
                            } else {
                                // Check if this div is a container for a div-based table
                                let is_div_table = classes.split_whitespace().any(|c| {
                                    let cleaned = c.trim_matches(|ch| ch == '\\' || ch == '"' || ch == '\'');
                                    cleaned == "table" || cleaned == "grid" || cleaned == "table-container"
                                });

                                if is_div_table {
                                    let mut rows = Vec::new();
                                    parse_div_table_rows(node, &mut rows);
                                    if !rows.is_empty() {
                                        blocks.push(ContentBlock::Table { rows });
                                        continue;
                                    }
                                }

                                // If it's a row div directly at block level:
                                let is_div_row = classes.split_whitespace().any(|c| {
                                    let cleaned = c.trim_matches(|ch| ch == '\\' || ch == '"' || ch == '\'');
                                    cleaned == "row" || cleaned == "table-row" || cleaned == "tr" || cleaned == "grid-row"
                                });

                                if is_div_row {
                                    let mut cells = Vec::new();
                                    parse_div_table_cells(node, &mut cells);
                                    if !cells.is_empty() {
                                        blocks.push(ContentBlock::Table {
                                            rows: vec![TableRow::Row { cells }],
                                        });
                                        continue;
                                    }
                                }

                                // Generic div — recurse into children
                                blocks.extend(parse_block_nodes(node.children()));
                            }
                        }

                        // ── Video ─────────────────────────────────────────────
                        "video" => {
                            let src = element
                                .attr("src")
                                .map(|s| clean_attribute(&s))
                                .or_else(|| {
                                    node.children().find_map(|c| {
                                        if let Node::Element(el) = c.value() {
                                            if el.name() == "source" {
                                                return el
                                                    .attr("src")
                                                    .map(|s| clean_attribute(&s));
                                            }
                                        }
                                        None
                                    })
                                })
                                .unwrap_or_default();

                            let poster = element.attr("poster").map(|s| clean_attribute(&s));
                            blocks.push(ContentBlock::Video { src, poster });
                        }

                        // ── Audio ─────────────────────────────────────────────
                        "audio" => {
                            let src = element
                                .attr("src")
                                .map(|s| clean_attribute(&s))
                                .or_else(|| {
                                    node.children().find_map(|c| {
                                        if let Node::Element(el) = c.value() {
                                            if el.name() == "source" {
                                                return el
                                                    .attr("src")
                                                    .map(|s| clean_attribute(&s));
                                            }
                                        }
                                        None
                                    })
                                })
                                .unwrap_or_default();

                            blocks.push(ContentBlock::Audio { src });
                        }

                        // ── Iframe embed ──────────────────────────────────────
                        "iframe" => {
                            let src = element
                                .attr("src")
                                .map(|s| clean_attribute(&s))
                                .unwrap_or_default();
                            let title = element.attr("title").map(|s| clean_attribute(&s));
                            blocks.push(ContentBlock::Embed { src, title });
                        }

                        // ── Generic container — recurse ───────────────────────
                        _ => {
                            blocks.extend(parse_block_nodes(node.children()));
                        }
                    }
                } else {
                    // Inline element at block level — accumulate into pending paragraph
                    parse_inline_node(node, &mut pending_inlines);
                }
            }

            _ => {}
        }
    }

    flush(&mut pending_inlines, &mut blocks);
    blocks
}

// ─────────────────────────────────────────────────────────────────────────────
// List item parser (handles nested <ul>/<ol> inside <li>)
// ─────────────────────────────────────────────────────────────────────────────

fn parse_list_items(list_node: NodeRef<Node>) -> Vec<ListItem> {
    let mut items = Vec::new();

    for child in list_node.children() {
        if let Node::Element(el) = child.value() {
            if el.name() == "li" {
                let mut inline_children: Vec<InlineNode> = Vec::new();
                let mut nested_blocks: Vec<ContentBlock> = Vec::new();

                for li_child in child.children() {
                    match li_child.value() {
                        Node::Element(li_el) => {
                            if matches!(li_el.name(), "ul" | "ol") {
                                // Nested list → becomes a nested ContentBlock::List
                                let ordered = li_el.name() == "ol";
                                let nested_items = parse_list_items(li_child);
                                nested_blocks.push(ContentBlock::List {
                                    ordered,
                                    items: nested_items,
                                });
                            } else {
                                parse_inline_node(li_child, &mut inline_children);
                            }
                        }
                        Node::Text(t) => {
                            inline_children.push(InlineNode::Text {
                                text: t.to_string(),
                            });
                        }
                        _ => {}
                    }
                }

                normalize_inline_nodes(&mut inline_children);
                items.push(ListItem::Item {
                    children: inline_children,
                    nested: nested_blocks,
                });
            }
        }
    }

    items
}

// ─────────────────────────────────────────────────────────────────────────────
// Definition list parser
// ─────────────────────────────────────────────────────────────────────────────

fn parse_definition_list(dl_node: NodeRef<Node>) -> Vec<DefinitionItem> {
    let mut items: Vec<DefinitionItem> = Vec::new();
    let mut current_term: Option<Vec<InlineNode>> = None;

    for child in dl_node.children() {
        if let Node::Element(el) = child.value() {
            match el.name() {
                "dt" => {
                    let mut term = parse_inline_nodes(child.children());
                    normalize_inline_nodes(&mut term);
                    current_term = Some(term);
                }
                "dd" => {
                    let mut definition = parse_inline_nodes(child.children());
                    normalize_inline_nodes(&mut definition);
                    let term = current_term.take().unwrap_or_default();
                    items.push(DefinitionItem { term, definition });
                }
                _ => {}
            }
        }
    }

    items
}

// ─────────────────────────────────────────────────────────────────────────────
// Inline parser
// ─────────────────────────────────────────────────────────────────────────────

/// Parse all children of a node as inline content.
fn parse_inline_nodes<'a, I>(nodes: I) -> Vec<InlineNode>
where
    I: IntoIterator<Item = NodeRef<'a, Node>>,
{
    let mut acc = Vec::new();
    for node in nodes {
        parse_inline_node(node, &mut acc);
    }
    acc
}

/// Map a single DOM node to zero or more [`InlineNode`]s and appends them to the buffer.
fn parse_inline_node(node: NodeRef<Node>, acc: &mut Vec<InlineNode>) {
    match node.value() {
        Node::Text(t) => acc.push(InlineNode::Text {
            text: t.to_string(),
        }),

        Node::Element(el) => match el.name() {
            "strong" | "b" => acc.push(InlineNode::Bold {
                children: parse_inline_nodes(node.children()),
            }),

            "em" | "i" => acc.push(InlineNode::Italic {
                children: parse_inline_nodes(node.children()),
            }),

            "a" => {
                let url = el
                    .attr("href")
                    .map(|s| clean_attribute(&s))
                    .unwrap_or_default();
                acc.push(InlineNode::Link {
                    url,
                    children: parse_inline_nodes(node.children()),
                });
            }

            "code" => acc.push(InlineNode::InlineCode {
                text: collect_text(node),
            }),

            "br" => acc.push(InlineNode::Break {}),

            // Any other inline wrapper (span, abbr, mark, …) — recurse into children
            _ => {
                for child in node.children() {
                    parse_inline_node(child, acc);
                }
            }
        },

        _ => {}
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Whitespace Normalization & Cleanup Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn normalize_inline_nodes(nodes: &mut Vec<InlineNode>) {
    if nodes.is_empty() {
        return;
    }

    let mut normalized = Vec::with_capacity(nodes.len());

    for node in nodes.drain(..) {
        match node {
            InlineNode::Text { text } => {
                let collapsed = collapse_whitespace(&text);
                if collapsed.is_empty() {
                    continue;
                }
                
                if let Some(InlineNode::Text { text: last_text }) = normalized.last_mut() {
                    let combined = format!("{}{}", last_text, collapsed);
                    *last_text = collapse_whitespace(&combined);
                } else {
                    normalized.push(InlineNode::Text { text: collapsed });
                }
            }
            InlineNode::Bold { mut children } => {
                normalize_inline_nodes(&mut children);
                if !children.is_empty() {
                    normalized.push(InlineNode::Bold { children });
                }
            }
            InlineNode::Italic { mut children } => {
                normalize_inline_nodes(&mut children);
                if !children.is_empty() {
                    normalized.push(InlineNode::Italic { children });
                }
            }
            InlineNode::Link { url, mut children } => {
                normalize_inline_nodes(&mut children);
                normalized.push(InlineNode::Link { url, children });
            }
            InlineNode::InlineCode { text } => {
                normalized.push(InlineNode::InlineCode { text: text.trim().to_string() });
            }
            InlineNode::Break {} => {
                normalized.push(InlineNode::Break {});
            }
        }
    }

    // Trim leading whitespace from the sequence
    if let Some(InlineNode::Text { text }) = normalized.first_mut() {
        let trimmed = text.trim_start();
        if trimmed.is_empty() {
            normalized.remove(0);
        } else {
            *text = trimmed.to_string();
        }
    }

    // Trim trailing whitespace from the sequence
    if let Some(InlineNode::Text { text }) = normalized.last_mut() {
        let trimmed = text.trim_end();
        if trimmed.is_empty() {
            normalized.pop();
        } else {
            *text = trimmed.to_string();
        }
    }

    *nodes = normalized;
}

fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_whitespace = false;

    for c in s.chars() {
        if c.is_whitespace() {
            if !in_whitespace {
                result.push(' ');
                in_whitespace = true;
            }
        } else {
            result.push(c);
            in_whitespace = false;
        }
    }

    result
}

// ─────────────────────────────────────────────────────────────────────────────
// Div-based table helper parsers
// ─────────────────────────────────────────────────────────────────────────────

fn parse_div_table_rows<'a>(node: NodeRef<'a, Node>, rows: &mut Vec<TableRow>) {
    for child in node.children() {
        if let Node::Element(el) = child.value() {
            let classes = el.attr("class").unwrap_or("");
            let is_row = classes.split_whitespace().any(|c| {
                let cleaned = c.trim_matches(|ch| ch == '\\' || ch == '"' || ch == '\'');
                cleaned == "row" || cleaned == "table-row" || cleaned == "tr" || cleaned == "grid-row"
            }) || el.name() == "tr";

            if is_row {
                let mut cells = Vec::new();
                parse_div_table_cells(child, &mut cells);
                if !cells.is_empty() {
                    rows.push(TableRow::Row { cells });
                }
            } else {
                // Recurse to find rows inside inner divs (e.g. tbody-like wrappers)
                parse_div_table_rows(child, rows);
            }
        }
    }
}

fn parse_div_table_cells<'a>(node: NodeRef<'a, Node>, cells: &mut Vec<TableCell>) {
    for child in node.children() {
        if let Node::Element(el) = child.value() {
            let classes = el.attr("class").unwrap_or("");
            let is_cell = classes.split_whitespace().any(|c| {
                let cleaned = c.trim_matches(|ch| ch == '\\' || ch == '"' || ch == '\'');
                cleaned == "col" || cleaned == "cell" || cleaned == "table-cell" || cleaned == "td" || cleaned == "th" || cleaned == "grid-cell"
            }) || matches!(el.name(), "td" | "th");

            if is_cell {
                let mut children = parse_inline_nodes(child.children());
                normalize_inline_nodes(&mut children);
                cells.push(TableCell::Cell { children });
            } else {
                // Recurse to find cells
                parse_div_table_cells(child, cells);
            }
        }
    }
}
