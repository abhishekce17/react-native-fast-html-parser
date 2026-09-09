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
            | "picture"
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
fn extract_best_url_from_srcset(srcset: &str) -> Option<String> {
    let cleaned = srcset.trim_matches(|c| c == '\\' || c == '"' || c == '\'');
    let mut best_url: Option<String> = None;
    let mut best_width: u32 = 0;

    for candidate in cleaned.split(',') {
        let trimmed = candidate.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if let Some(&first) = parts.first() {
            let url = clean_attribute(first);
            if url.is_empty() {
                continue;
            }
            if best_url.is_none() {
                best_url = Some(url.clone());
            }
            if parts.len() > 1 {
                if let Some(descriptor) = parts.get(1) {
                    if let Some(w) = descriptor.strip_suffix('w') {
                        if let Ok(width_val) = w.parse::<u32>() {
                            if width_val > best_width {
                                best_width = width_val;
                                best_url = Some(url);
                            }
                        }
                    } else if let Some(x) = descriptor.strip_suffix('x') {
                        if let Ok(density_val) = x.parse::<f32>() {
                            let width_val = (density_val * 1000.0) as u32;
                            if width_val > best_width {
                                best_width = width_val;
                                best_url = Some(url);
                            }
                        }
                    }
                }
            }
        }
    }
    best_url
}

fn extract_img_attributes(el: &scraper::node::Element) -> (String, Option<String>) {
    let mut raw_url = el
        .attr("src")
        .map(|s| clean_attribute(&s))
        .unwrap_or_default();

    let is_placeholder = raw_url.is_empty()
        || (raw_url.starts_with("data:image/") && raw_url.len() < 300);

    if is_placeholder {
        if let Some(lazy) = el
            .attr("data-src")
            .or_else(|| el.attr("data-original"))
            .or_else(|| el.attr("data-url"))
            .or_else(|| el.attr("data-lazy-src"))
            .or_else(|| el.attr("data-actualsrc"))
            .or_else(|| el.attr("data-orig-file"))
            .or_else(|| el.attr("data-full-url"))
            .or_else(|| el.attr("data-hi-res-src"))
        {
            let cleaned = clean_attribute(lazy);
            if !cleaned.is_empty() {
                raw_url = cleaned;
            }
        }
    }

    if is_placeholder || raw_url.is_empty() {
        if let Some(srcset) = el.attr("srcset").or_else(|| el.attr("data-srcset")) {
            if let Some(url_from_srcset) = extract_best_url_from_srcset(srcset) {
                raw_url = url_from_srcset;
            }
        }
    }

    let alt = el.attr("alt").map(|s| clean_attribute(&s));
    (raw_url, alt)
}

fn find_image_in_subtree(node: NodeRef<Node>) -> Option<(String, Option<String>, Option<String>)> {
    if let Node::Element(el) = node.value() {
        match el.name() {
            "img" => {
                let (url, alt) = extract_img_attributes(el);
                if !url.is_empty() {
                    return Some((url, alt, None));
                }
            }
            "picture" => {
                for child in node.children() {
                    if let Node::Element(child_el) = child.value() {
                        if child_el.name() == "source" {
                            if let Some(srcset) = child_el.attr("srcset").or_else(|| child_el.attr("src")) {
                                if let Some(url) = extract_best_url_from_srcset(srcset) {
                                    let alt = child_el.attr("alt").map(|s| clean_attribute(&s));
                                    return Some((url, alt, None));
                                }
                            }
                        } else if child_el.name() == "img" {
                            let (url, alt) = extract_img_attributes(child_el);
                            if !url.is_empty() {
                                return Some((url, alt, None));
                            }
                        }
                    }
                }
            }
            "a" => {
                let link_url = el.attr("href").map(|s| clean_attribute(&s));
                for child in node.children() {
                    if let Some((url, alt, _)) = find_image_in_subtree(child) {
                        return Some((url, alt, link_url));
                    }
                }
            }
            "span" | "div" | "figure" | "noscript" | "p" => {
                for child in node.children() {
                    if let Some((url, alt, link)) = find_image_in_subtree(child) {
                        return Some((url, alt, link));
                    }
                }
            }
            _ => {}
        }
    }
    None
}

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
                        // ── Paragraph (with native embedded/wrapped image support) ──
                        "p" => {
                            let mut p_inlines: Vec<InlineNode> = Vec::new();
                            let mut had_image = false;

                            for child in node.children() {
                                if let Some((img_url, img_alt, img_link)) = find_image_in_subtree(child) {
                                    had_image = true;
                                    normalize_inline_nodes(&mut p_inlines);
                                    if !p_inlines.is_empty() {
                                        blocks.push(ContentBlock::Paragraph {
                                            children: p_inlines.clone(),
                                        });
                                        p_inlines.clear();
                                    }
                                    blocks.push(ContentBlock::Image {
                                        url: img_url,
                                        alt: img_alt,
                                        link_url: img_link,
                                    });
                                } else {
                                    parse_inline_node(child, &mut p_inlines);
                                }
                            }

                            normalize_inline_nodes(&mut p_inlines);
                            if !had_image || !p_inlines.is_empty() {
                                blocks.push(ContentBlock::Paragraph {
                                    children: p_inlines,
                                });
                            }
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
                            let (url, alt) = extract_img_attributes(element);
                            blocks.push(ContentBlock::Image {
                                url,
                                alt,
                                link_url: None,
                            });
                        }

                        // ── Picture container ─────────────────────────────────
                        "picture" => {
                            if let Some((url, alt, link_url)) = find_image_in_subtree(node) {
                                blocks.push(ContentBlock::Image {
                                    url,
                                    alt,
                                    link_url,
                                });
                            }
                        }

                        // ── Figure (image + optional caption + optional link) ──
                        "figure" => {
                            let mut url = String::new();
                            let mut alt: Option<String> = None;
                            let mut caption: Option<String> = None;
                            let mut link_url: Option<String> = None;

                            for child in node.children() {
                                if let Node::Element(el) = child.value() {
                                    if el.name() == "figcaption" {
                                        let text = collect_text(child).trim().to_string();
                                        if !text.is_empty() {
                                            caption = Some(text);
                                        }
                                    } else if let Some((img_url, img_alt, img_link)) = find_image_in_subtree(child) {
                                        if !img_url.is_empty() {
                                            url = img_url;
                                            alt = img_alt;
                                            if img_link.is_some() {
                                                link_url = img_link;
                                            }
                                        }
                                    }
                                }
                            }
                            blocks.push(ContentBlock::Figure {
                                url,
                                alt,
                                caption,
                                link_url,
                            });
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

            // Ignore non-content nodes
            "script" | "style" | "svg" | "noscript" | "template" | "head" | "meta" => {}

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

// ─────────────────────────────────────────────────────────────────────────────
// Unit Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ContentBlock, DefinitionItem, InlineNode, ListItem, TableCell, TableRow};

    // ── Group 1: Basic & Empty Inputs ──────────────────────────────────────────

    #[test]
    fn test_parse_empty_and_whitespace() {
        assert!(parse_html("").is_empty());
        assert!(parse_html("   \n\t  ").is_empty());
        let empty_p = parse_html("<p></p>");
        assert_eq!(empty_p.len(), 1);
        if let ContentBlock::Paragraph { children } = &empty_p[0] {
            assert!(children.is_empty());
        }
        assert!(parse_html("<div>   </div>").is_empty());
    }

    #[test]
    fn test_parse_paragraphs_and_inlines() {
        let html = "<p>Hello <b>bold</b> <i>italic</i> <a href=\"https://example.com\">link</a> <code>code</code></p>";
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);

        if let ContentBlock::Paragraph { children } = &blocks[0] {
            assert!(!children.is_empty());
            let has_bold = children.iter().any(|c| matches!(c, InlineNode::Bold { .. }));
            let has_italic = children.iter().any(|c| matches!(c, InlineNode::Italic { .. }));
            let has_link = children.iter().any(|c| matches!(c, InlineNode::Link { .. }));
            let has_code = children.iter().any(|c| matches!(c, InlineNode::InlineCode { .. }));
            assert!(has_bold && has_italic && has_link && has_code);
        } else {
            panic!("Expected Paragraph block");
        }
    }

    #[test]
    fn test_parse_headings() {
        let html = "<h1>Heading 1</h1><h2>Heading 2</h2><h3>Heading 3</h3><h4>H4</h4><h5>H5</h5><h6>H6</h6>";
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 6);

        for (i, block) in blocks.iter().enumerate() {
            if let ContentBlock::Heading { level, children } = block {
                assert_eq!(*level, (i + 1) as u8);
                assert!(!children.is_empty());
            } else {
                panic!("Expected Heading block at index {i}");
            }
        }
    }

    #[test]
    fn test_headings_with_inline_formatting() {
        let html = "<h1>Title with <b>Bold</b> and <i>Italic</i> and <a href=\"https://test.com\">Link</a></h1>";
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Heading { level, children } = &blocks[0] {
            assert_eq!(*level, 1);
            assert!(children.len() >= 3);
            let has_bold = children.iter().any(|c| matches!(c, InlineNode::Bold { .. }));
            let has_italic = children.iter().any(|c| matches!(c, InlineNode::Italic { .. }));
            let has_link = children.iter().any(|c| matches!(c, InlineNode::Link { .. }));
            assert!(has_bold && has_italic && has_link);
        } else {
            panic!("Expected Heading");
        }
    }

    // ── Group 2: Complex Lists & Nesting ──────────────────────────────────────

    #[test]
    fn test_parse_nested_lists() {
        let html = r#"
            <ul>
                <li>Item 1</li>
                <li>Item 2
                    <ol>
                        <li>Sub 2.1</li>
                        <li>Sub 2.2</li>
                    </ol>
                </li>
                <li>Item 3</li>
            </ul>
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);

        if let ContentBlock::List { ordered, items } = &blocks[0] {
            assert!(!ordered);
            assert_eq!(items.len(), 3);
            let ListItem::Item { nested, .. } = &items[1];
            assert_eq!(nested.len(), 1);
            if let ContentBlock::List { ordered: sub_ordered, items: sub_items } = &nested[0] {
                assert!(sub_ordered);
                assert_eq!(sub_items.len(), 2);
            } else {
                panic!("Expected nested ordered list");
            }
        } else {
            panic!("Expected List block");
        }
    }

    #[test]
    fn test_deeply_nested_lists_4_levels() {
        let html = r#"
            <ul>
                <li>Level 1
                    <ol>
                        <li>Level 2
                            <ul>
                                <li>Level 3
                                    <ol>
                                        <li>Level 4 deepest</li>
                                    </ol>
                                </li>
                            </ul>
                        </li>
                    </ol>
                </li>
            </ul>
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::List { items, .. } = &blocks[0] {
            assert_eq!(items.len(), 1);
            let ListItem::Item { nested: l2, .. } = &items[0];
            assert_eq!(l2.len(), 1);
            if let ContentBlock::List { items: l2_items, .. } = &l2[0] {
                let ListItem::Item { nested: l3, .. } = &l2_items[0];
                assert_eq!(l3.len(), 1);
                if let ContentBlock::List { items: l3_items, .. } = &l3[0] {
                    let ListItem::Item { nested: l4, .. } = &l3_items[0];
                    assert_eq!(l4.len(), 1);
                }
            }
        }
    }

    #[test]
    fn test_list_items_with_inline_formatting_and_links() {
        let html = r#"
            <ul>
                <li>Item with <b>bold text</b></li>
                <li>Item with <a href="https://example.com">external link</a></li>
                <li>Item with <code>console.log()</code> code</li>
            </ul>
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::List { items, .. } = &blocks[0] {
            assert_eq!(items.len(), 3);
            let ListItem::Item { children: c0, .. } = &items[0];
            assert!(c0.iter().any(|c| matches!(c, InlineNode::Bold { .. })));
            let ListItem::Item { children: c1, .. } = &items[1];
            assert!(c1.iter().any(|c| matches!(c, InlineNode::Link { .. })));
            let ListItem::Item { children: c2, .. } = &items[2];
            assert!(c2.iter().any(|c| matches!(c, InlineNode::InlineCode { .. })));
        }
    }

    // ── Group 3: Code Blocks & Language Detection ──────────────────────────────

    #[test]
    fn test_parse_codeblock_language_detection() {
        let html = r#"
            <pre><code class="language-typescript">const x = 1;</code></pre>
            <pre class="lang-python"><code>def foo(): pass</code></pre>
            <div class="highlight rust"><pre><code>fn main() {}</code></pre></div>
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 3);

        if let ContentBlock::CodeBlock { language, code } = &blocks[0] {
            assert_eq!(language.as_deref(), Some("typescript"));
            assert_eq!(code, "const x = 1;");
        } else {
            panic!("Expected CodeBlock");
        }

        if let ContentBlock::CodeBlock { language, code } = &blocks[1] {
            assert_eq!(language.as_deref(), Some("python"));
            assert_eq!(code, "def foo(): pass");
        } else {
            panic!("Expected CodeBlock");
        }

        if let ContentBlock::CodeBlock { language, code } = &blocks[2] {
            assert_eq!(language.as_deref(), Some("rust"));
            assert_eq!(code, "fn main() {}");
        } else {
            panic!("Expected CodeBlock");
        }
    }

    #[test]
    fn test_codeblock_with_html_tags_inside() {
        let html = r#"<pre><code class="language-html">&lt;div class="box"&gt;&lt;span&gt;Text&lt;/span&gt;&lt;/div&gt;</code></pre>"#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::CodeBlock { language, code } = &blocks[0] {
            assert_eq!(language.as_deref(), Some("html"));
            assert!(code.contains("<div class=\"box\">"));
        } else {
            panic!("Expected CodeBlock");
        }
    }

    #[test]
    fn test_codeblock_bare_pre_without_code_tag() {
        let html = r#"<pre class="language-go">package main&#10;func main() {}</pre>"#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::CodeBlock { language, code } = &blocks[0] {
            assert_eq!(language.as_deref(), Some("go"));
            assert!(code.contains("package main"));
        } else {
            panic!("Expected CodeBlock");
        }
    }

    #[test]
    fn test_codeblock_with_inner_spans_syntax_tokens() {
        let html = r#"<pre><code class="language-javascript"><span class="kwd">const</span> <span class="var">name</span> = <span class="str">"Antigravity"</span>;</code></pre>"#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::CodeBlock { language, code } = &blocks[0] {
            assert_eq!(language.as_deref(), Some("javascript"));
            assert_eq!(code, "const name = \"Antigravity\";");
        } else {
            panic!("Expected CodeBlock");
        }
    }

    #[test]
    fn test_codeblock_with_quoted_escaped_attributes() {
        let html = r#"<pre class=\"language-cpp\"><code>#include &lt;iostream&gt;</code></pre>"#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::CodeBlock { language, code } = &blocks[0] {
            assert_eq!(language.as_deref(), Some("cpp"));
            assert!(code.contains("#include <iostream>"));
        }
    }

    // ── Group 4: Tables & Ragged Rows ──────────────────────────────────────────

    #[test]
    fn test_parse_tables() {
        let html = r#"
            <table>
                <thead>
                    <tr><th>Name</th><th>Role</th></tr>
                </thead>
                <tbody>
                    <tr><td>Alice</td><td>Engineer</td></tr>
                    <tr><td>Bob</td><td>Designer</td></tr>
                </tbody>
            </table>
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);

        if let ContentBlock::Table { rows } = &blocks[0] {
            assert_eq!(rows.len(), 3);
            let TableRow::Row { cells } = &rows[0];
            assert_eq!(cells.len(), 2);
        } else {
            panic!("Expected Table block");
        }
    }

    #[test]
    fn test_table_with_all_inline_types_in_cells() {
        let html = r#"
            <table>
                <tr>
                    <td><b>Bold</b> text</td>
                    <td><i>Italic</i> text</td>
                    <td><a href="https://example.com">Link</a></td>
                    <td><code>inline_code()</code></td>
                </tr>
            </table>
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Table { rows } = &blocks[0] {
            assert_eq!(rows.len(), 1);
            let TableRow::Row { cells } = &rows[0];
            assert_eq!(cells.len(), 4);
            let TableCell::Cell { children: c0 } = &cells[0];
            assert!(c0.iter().any(|c| matches!(c, InlineNode::Bold { .. })));
            let TableCell::Cell { children: c1 } = &cells[1];
            assert!(c1.iter().any(|c| matches!(c, InlineNode::Italic { .. })));
            let TableCell::Cell { children: c2 } = &cells[2];
            assert!(c2.iter().any(|c| matches!(c, InlineNode::Link { .. })));
            let TableCell::Cell { children: c3 } = &cells[3];
            assert!(c3.iter().any(|c| matches!(c, InlineNode::InlineCode { .. })));
        }
    }

    #[test]
    fn test_table_ragged_rows_and_empty_cells() {
        let html = r#"
            <table>
                <tr><td>Cell 1</td><td>Cell 2</td><td>Cell 3</td></tr>
                <tr><td>Only 1 Cell</td></tr>
                <tr><td></td><td>   </td></tr>
            </table>
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Table { rows } = &blocks[0] {
            assert_eq!(rows.len(), 3);
            let TableRow::Row { cells: r0 } = &rows[0];
            assert_eq!(r0.len(), 3);
            let TableRow::Row { cells: r1 } = &rows[1];
            assert_eq!(r1.len(), 1);
            let TableRow::Row { cells: r2 } = &rows[2];
            assert_eq!(r2.len(), 2);
        }
    }

    #[test]
    fn test_table_with_tfoot_and_caption() {
        let html = r#"
            <table>
                <caption>Quarterly Summary</caption>
                <thead><tr><th>Q</th><th>Revenue</th></tr></thead>
                <tbody><tr><td>Q1</td><td>$100K</td></tr></tbody>
                <tfoot><tr><td>Total</td><td>$100K</td></tr></tfoot>
            </table>
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Table { rows } = &blocks[0] {
            assert_eq!(rows.len(), 3); // thead, tbody, tfoot
        }
    }

    // ── Group 5: Blockquotes & Nested Structures ──────────────────────────────

    #[test]
    fn test_parse_blockquotes() {
        let html = "<blockquote><p>Quote line 1</p><p>Quote line 2</p></blockquote>";
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);

        if let ContentBlock::Quote { children } = &blocks[0] {
            assert_eq!(children.len(), 2);
        } else {
            panic!("Expected Quote block");
        }
    }

    #[test]
    fn test_nested_blockquotes_multi_level() {
        let html = r#"
            <blockquote>
                <p>Level 1 quote</p>
                <blockquote>
                    <p>Level 2 quote</p>
                    <blockquote>
                        <p>Level 3 deepest quote</p>
                    </blockquote>
                </blockquote>
            </blockquote>
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Quote { children: l1 } = &blocks[0] {
            assert_eq!(l1.len(), 2);
            if let ContentBlock::Quote { children: l2 } = &l1[1] {
                assert_eq!(l2.len(), 2);
                assert!(matches!(&l2[1], ContentBlock::Quote { .. }));
            } else {
                panic!("Expected nested quote");
            }
        }
    }

    #[test]
    fn test_blockquote_containing_headings_and_lists() {
        let html = r#"
            <blockquote>
                <h2>Quoted Section</h2>
                <p>Paragraph inside quote</p>
                <ul><li>List in quote</li></ul>
            </blockquote>
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Quote { children } = &blocks[0] {
            assert_eq!(children.len(), 3);
            assert!(matches!(&children[0], ContentBlock::Heading { .. }));
            assert!(matches!(&children[1], ContentBlock::Paragraph { .. }));
            assert!(matches!(&children[2], ContentBlock::List { .. }));
        }
    }

    // ── Group 6: Media, Figures & Embeds ──────────────────────────────────────

    #[test]
    fn test_paragraph_with_wrapped_and_linked_images() {
        let html = r#"
            <p><img src="https://example.com/standalone.png" alt="Standalone in p" /></p>
            <p><a href="https://example.com/full"><img src="https://example.com/linked.png" alt="Linked in p a" /></a></p>
            <p>Intro text <img src="https://example.com/inline.png" alt="Middle img" /> Outro text</p>
            <img data-src="https://example.com/lazy.png" alt="Lazy data-src" />
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 6);

        match &blocks[0] {
            ContentBlock::Image { url, alt, link_url } => {
                assert_eq!(url, "https://example.com/standalone.png");
                assert_eq!(alt.as_deref(), Some("Standalone in p"));
                assert_eq!(link_url.as_deref(), None);
            }
            _ => panic!("Expected Image for wrapped p > img"),
        }

        match &blocks[1] {
            ContentBlock::Image { url, alt, link_url } => {
                assert_eq!(url, "https://example.com/linked.png");
                assert_eq!(alt.as_deref(), Some("Linked in p a"));
                assert_eq!(link_url.as_deref(), Some("https://example.com/full"));
            }
            _ => panic!("Expected Image for linked p > a > img"),
        }

        match &blocks[2] {
            ContentBlock::Paragraph { children } => {
                match &children[0] {
                    InlineNode::Text { text } => assert_eq!(text, "Intro text"),
                    _ => panic!("Expected Text node"),
                }
            }
            _ => panic!("Expected Paragraph intro"),
        }

        match &blocks[3] {
            ContentBlock::Image { url, alt, link_url } => {
                assert_eq!(url, "https://example.com/inline.png");
                assert_eq!(alt.as_deref(), Some("Middle img"));
                assert_eq!(link_url.as_deref(), None);
            }
            _ => panic!("Expected Image middle"),
        }

        match &blocks[4] {
            ContentBlock::Paragraph { children } => {
                match &children[0] {
                    InlineNode::Text { text } => assert_eq!(text, "Outro text"),
                    _ => panic!("Expected Text node"),
                }
            }
            _ => panic!("Expected Paragraph outro"),
        }

        match &blocks[5] {
            ContentBlock::Image { url, alt, link_url } => {
                assert_eq!(url, "https://example.com/lazy.png");
                assert_eq!(alt.as_deref(), Some("Lazy data-src"));
                assert_eq!(link_url.as_deref(), None);
            }
            _ => panic!("Expected Image lazy data-src"),
        }
    }


    #[test]
    fn test_parse_media_and_embeds() {
        let html = r#"
            <img src="https://example.com/pic.jpg" alt="A photo" />
            <figure>
                <img src="https://example.com/fig.png" alt="Figure photo" />
                <figcaption>Figure caption text</figcaption>
            </figure>
            <video src="https://example.com/video.mp4" poster="https://example.com/poster.jpg"></video>
            <audio src="https://example.com/audio.mp3"></audio>
            <iframe src="https://youtube.com/embed/xyz" title="YouTube video"></iframe>
            <hr />
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 6);

        match &blocks[0] {
            ContentBlock::Image { url, alt, .. } => {
                assert_eq!(url, "https://example.com/pic.jpg");
                assert_eq!(alt.as_deref(), Some("A photo"));
            }
            _ => panic!("Expected Image"),
        }

        match &blocks[1] {
            ContentBlock::Figure { url, alt, caption, .. } => {
                assert_eq!(url, "https://example.com/fig.png");
                assert_eq!(alt.as_deref(), Some("Figure photo"));
                assert_eq!(caption.as_deref(), Some("Figure caption text"));
            }
            _ => panic!("Expected Figure"),
        }

        match &blocks[2] {
            ContentBlock::Video { src, poster } => {
                assert_eq!(src, "https://example.com/video.mp4");
                assert_eq!(poster.as_deref(), Some("https://example.com/poster.jpg"));
            }
            _ => panic!("Expected Video"),
        }

        match &blocks[3] {
            ContentBlock::Audio { src } => {
                assert_eq!(src, "https://example.com/audio.mp3");
            }
            _ => panic!("Expected Audio"),
        }

        match &blocks[4] {
            ContentBlock::Embed { src, title } => {
                assert_eq!(src, "https://youtube.com/embed/xyz");
                assert_eq!(title.as_deref(), Some("YouTube video"));
            }
            _ => panic!("Expected Embed"),
        }

        match &blocks[5] {
            ContentBlock::Separator {} => {}
            _ => panic!("Expected Separator"),
        }
    }

    #[test]
    fn test_video_with_source_tag() {
        let html = r#"<video poster="cover.jpg"><source src="https://stream.io/video.m3u8" type="application/x-mpegURL"></video>"#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Video { src, poster } = &blocks[0] {
            assert_eq!(src, "https://stream.io/video.m3u8");
            assert_eq!(poster.as_deref(), Some("cover.jpg"));
        } else {
            panic!("Expected Video");
        }
    }

    #[test]
    fn test_audio_with_source_tag() {
        let html = r#"<audio><source src="https://stream.io/audio.mp3" type="audio/mpeg"></audio>"#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Audio { src } = &blocks[0] {
            assert_eq!(src, "https://stream.io/audio.mp3");
        } else {
            panic!("Expected Audio");
        }
    }

    #[test]
    fn test_figure_with_formatted_caption() {
        let html = r#"
            <figure>
                <img src="chart.png" alt="Sales Growth" />
                <figcaption>Chart showing <b>2026</b> performance metrics with <i>positive</i> ROI.</figcaption>
            </figure>
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Figure { url, caption, .. } = &blocks[0] {
            assert_eq!(url, "chart.png");
            assert!(caption.as_ref().unwrap().contains("2026"));
        }
    }

    // ── Group 7: Definition Lists ─────────────────────────────────────────────

    #[test]
    fn test_parse_definition_list() {
        let html = "<dl><dt>Term 1</dt><dd>Def 1</dd><dt>Term 2</dt><dd>Def 2</dd></dl>";
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);

        if let ContentBlock::DefinitionList { items } = &blocks[0] {
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected DefinitionList");
        }
    }

    #[test]
    fn test_definition_list_multiple_terms_and_defs() {
        let html = r#"
            <dl>
                <dt>Rust</dt>
                <dt>Rust-lang</dt>
                <dd>A language empowering everyone to build reliable and efficient software.</dd>
                <dt>JSI</dt>
                <dd>JavaScript Interface for C++ HostObjects.</dd>
                <dd>Direct memory bridging without JSON serialization overhead.</dd>
            </dl>
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::DefinitionList { items } = &blocks[0] {
            assert!(items.len() >= 2);
            let first_item: &DefinitionItem = &items[0];
            assert!(!first_item.definition.is_empty());
        }
    }

    // ── Group 8: Malformed HTML, Overlapping Tags & Tag Recovery ────────────────

    #[test]
    fn test_malformed_html_recovery() {
        let html = "<p>First <b>bold <i>italic without closing <p>Second paragraph <div>Inside div <li>item without ul";
        let blocks = parse_html(html);
        assert!(blocks.len() >= 2);
    }

    #[test]
    fn test_overlapping_inline_tags_acid_test() {
        let html = "<p>Text with <b>bold <i>bold-and-italic</b> italic-only</i> regular text</p>";
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Paragraph { children } = &blocks[0] {
            assert!(!children.is_empty());
        }
    }

    #[test]
    fn test_deeply_nested_unclosed_tags() {
        let html = "<section><div><article><div><p><span><b><i>Deeply unclosed text with no ending tags";
        let blocks = parse_html(html);
        assert!(!blocks.is_empty());
    }

    #[test]
    fn test_stray_table_elements_outside_table() {
        let html = "<tr><td>Floating Cell 1</td><td>Floating Cell 2</td></tr>";
        let blocks = parse_html(html);
        assert!(!blocks.is_empty());
    }

    #[test]
    fn test_stray_list_items_outside_list() {
        let html = "<li>Stray item A</li><li>Stray item B</li>";
        let blocks = parse_html(html);
        assert!(!blocks.is_empty());
    }

    #[test]
    fn test_script_style_svg_stripping() {
        let html = "<p>Visible text <script>alert('xss');</script><style>body { color: red; }</style><svg><path d=\"M0 0\"/></svg> and trailing visible text</p>";
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Paragraph { children } = &blocks[0] {
            let combined_text: String = children.iter().map(|c| match c {
                InlineNode::Text { text } => text.clone(),
                _ => String::new(),
            }).collect();
            assert!(combined_text.contains("Visible text"));
            assert!(combined_text.contains("trailing visible text"));
            assert!(!combined_text.contains("alert('xss')"));
            assert!(!combined_text.contains("color: red"));
        }
    }

    #[test]
    fn test_html_comments_and_cdata_stripping() {
        let html = "<!-- Comment 1 --><div><!-- Inner comment --><p>Clean <!-- inline comment -->content</p></div>";
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Paragraph { children } = &blocks[0] {
            let text = match &children[0] {
                InlineNode::Text { text } => text,
                _ => "",
            };
            assert!(text.contains("Clean content") || text.contains("Clean"));
        }
    }

    // ── Group 9: HTML Entities, Unicode, RTL & CJK ─────────────────────────────

    #[test]
    fn test_html_entities_complex_decoding() {
        let html = "<p>&amp; &lt; &gt; &quot; &#39; &nbsp; &copy; &euro; &mdash; &ndash; &#x1F680;</p>";
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Paragraph { children } = &blocks[0] {
            assert!(!children.is_empty());
            let text = match &children[0] {
                InlineNode::Text { text } => text,
                _ => "",
            };
            assert!(text.contains('&'));
            assert!(text.contains('<'));
            assert!(text.contains('>'));
        }
    }

    #[test]
    fn test_unicode_emojis_rtl_and_cjk_scripts() {
        let html = r#"
            <h1>Unicode Editorial 🚀 🎉</h1>
            <p dir="rtl" lang="he">שלום עולם - זהו מבחן עברית</p>
            <p dir="rtl" lang="ar">مرحبا بالعالم - اختبار اللغة العربية</p>
            <p lang="ja">こんにちは世界！React Native パルサー</p>
            <p lang="zh">你好世界，高效的 HTML 解析引擎</p>
            <p lang="ko">안녕하세요 세계, 고성능 파서</p>
            <p lang="ru">Привет, мир! Быстрый парсер HTML</p>
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 7);
        assert!(matches!(&blocks[0], ContentBlock::Heading { .. }));
        for b in &blocks[1..] {
            assert!(matches!(b, ContentBlock::Paragraph { .. }));
        }
    }

    #[test]
    fn test_matrix_data_and_aria_attributes() {
        let html = r#"<p id="p-1" class="intro" data-track="123" aria-label="Intro text" dir="rtl" lang="ar">مرحبا</p>"#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Paragraph { children } = &blocks[0] {
            assert_eq!(children.len(), 1);
        } else {
            panic!("Expected Paragraph block");
        }
    }

    #[test]
    fn test_matrix_nested_inline_formatting() {
        let html = "<p>Text with <b>bold <i>italic <u>underline <s>strike <a href=\"https://test.com\">link</a></s></u></i></b> and <code>code</code><br/>next line</p>";
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Paragraph { children } = &blocks[0] {
            assert!(children.len() >= 3);
        } else {
            panic!("Expected Paragraph block");
        }
    }

    #[test]
    fn test_excessive_whitespace_tabs_newlines_collapse() {
        let html = "   <p>   Multiple \n\n\t  spaces    inside   <b>   bold   text  </b>  </p>   ";
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Paragraph { children } = &blocks[0] {
            let first = &children[0];
            if let InlineNode::Text { text } = first {
                assert!(!text.starts_with("   "));
            }
        }
    }

    // ── Group 10: Modern Semantic Containers & Div Grids ────────────────────────

    #[test]
    fn test_matrix_div_grid_table() {
        let html = r#"
            <div class="table-container grid">
                <div class="table-row">
                    <div class="table-cell">Header 1</div>
                    <div class="table-cell">Header 2</div>
                </div>
                <div class="table-row">
                    <div class="table-cell">Data 1</div>
                    <div class="table-cell">Data 2</div>
                </div>
            </div>
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Table { rows } = &blocks[0] {
            assert_eq!(rows.len(), 2);
        } else {
            panic!("Expected Table block for div-grid");
        }
    }

    #[test]
    fn test_semantic_html5_containers() {
        let html = r#"
            <article>
                <header>
                    <h1>Article Title</h1>
                </header>
                <section>
                    <p>Section 1 text</p>
                </section>
                <aside>
                    <p>Sidebar notice</p>
                </aside>
                <footer>
                    <p>Copyright 2026</p>
                </footer>
            </article>
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 4);
        assert!(matches!(&blocks[0], ContentBlock::Heading { .. }));
        assert!(matches!(&blocks[1], ContentBlock::Paragraph { .. }));
        assert!(matches!(&blocks[2], ContentBlock::Paragraph { .. }));
        assert!(matches!(&blocks[3], ContentBlock::Paragraph { .. }));
    }

    #[test]
    fn test_mixed_inline_and_blocks_in_div() {
        let html = "<div>Direct text before <h2>Header</h2> Direct text between <p>Paragraph</p> Direct text after</div>";
        let blocks = parse_html(html);
        assert!(blocks.len() >= 4);
    }

    #[test]
    fn test_consecutive_horizontal_rules() {
        let html = "<hr /><hr /><p>Middle text</p><hr />";
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 4);
        assert!(matches!(&blocks[0], ContentBlock::Separator {}));
        assert!(matches!(&blocks[1], ContentBlock::Separator {}));
        assert!(matches!(&blocks[2], ContentBlock::Paragraph { .. }));
        assert!(matches!(&blocks[3], ContentBlock::Separator {}));
    }

    #[test]
    fn test_link_with_nested_bold_and_italic() {
        let html = r#"<p><a href="https://nitro.margelo.com"><b>Bold</b> and <i>Italic</i> within anchor</a></p>"#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Paragraph { children } = &blocks[0] {
            assert_eq!(children.len(), 1);
            if let InlineNode::Link { url, children: link_inlines } = &children[0] {
                assert_eq!(url, "https://nitro.margelo.com");
                assert!(link_inlines.iter().any(|c| matches!(c, InlineNode::Bold { .. })));
                assert!(link_inlines.iter().any(|c| matches!(c, InlineNode::Italic { .. })));
            } else {
                panic!("Expected Link node");
            }
        }
    }

    #[test]
    fn test_json_roundtrip_fidelity_complex_document() {
        let html = r#"
            <h1>Full Editorial Suite</h1>
            <p>Intro with <b>bold</b>, <i>italic</i>, and <a href="https://example.com">link</a>.</p>
            <ul>
                <li>Bullet 1</li>
                <li>Bullet 2 with <ol><li>Sub 2.1</li></ol></li>
            </ul>
            <blockquote><p>Quote content</p></blockquote>
            <pre><code class="language-rust">fn main() { println!("Hello"); }</code></pre>
            <img src="https://example.com/image.png" alt="Test" />
            <table>
                <tr><th>Col 1</th><th>Col 2</th></tr>
                <tr><td>Val 1</td><td>Val 2</td></tr>
            </table>
            <hr />
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 8);

        // Serialize to JSON
        let json_str = serde_json::to_string(&blocks).expect("Serialization failed");
        assert!(!json_str.is_empty());

        // Deserialize from JSON and verify block count
        let parsed_json: serde_json::Value = serde_json::from_str(&json_str).expect("Deserialization failed");
        assert!(parsed_json.is_array());
        assert_eq!(parsed_json.as_array().unwrap().len(), 8);
    }

    #[test]
    fn test_picture_and_lazy_images() {
        let html = r#"
            <picture>
                <source srcset="https://example.com/receipt-small.webp 400w, https://example.com/receipt-large.webp 1200w" />
                <img src="data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7" data-src="https://example.com/receipt.png" alt="Receipt" />
            </picture>
            <p>
                <span class="img-wrap">
                    <img srcset="https://example.com/thumb.jpg 300w, https://example.com/full.jpg 1000w" alt="Full Image" />
                </span>
            </p>
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 2);

        match &blocks[0] {
            ContentBlock::Image { url, .. } => {
                assert!(url.contains("receipt"));
            }
            _ => panic!("Expected Image block"),
        }

        match &blocks[1] {
            ContentBlock::Image { url, .. } => {
                assert_eq!(url, "https://example.com/full.jpg");
            }
            _ => panic!("Expected Image block"),
        }
    }

    #[test]
    fn test_devto_receipt_image() {
        let html = r#"
            <p>The savings are also auditable rather than a vibe. Every request writes to a local ledger with its real cost and the counterfactual of what a frontier-only run would have cost, so the report is arithmetic you can check. A real receipt from one of my sessions:</p>
            <p><a href="https://media2.dev.to/dynamic/image/width=800%2Cheight=%2Cfit=scale-down%2Cgravity=auto%2Cformat=auto/https%3A%2F%2Fdev-to-uploads.s3.us-east-2.amazonaws.com%2Fuploads%2Farticles%2Ffo0hpgcvhvigtpjof6g0.png" class="article-body-image-wrapper"><img src="https://media2.dev.to/dynamic/image/width=800%2Cheight=%2Cfit=scale-down%2Cgravity=auto%2Cformat=auto/https%3A%2F%2Fdev-to-uploads.s3.us-east-2.amazonaws.com%2Fuploads%2Farticles%2Ffo0hpgcvhvigtpjof6g0.png" alt="coding-agent-router savings report: $0.12 spent vs $1.23 frontier-only, 90% saved on that run" loading="lazy" width="800" height="336"></a></p>
            <p>That run happened to route six requests to the cheap tier with zero frontier usage, which is realistic for a stretch of mechanical work and not representative of every session. A debugging-heavy afternoon escalates early and saves less. The honest claim is not a fixed percentage, it is that the routine majority of your calls stop being billed at frontier rates.</p>
        "#;
        let blocks = parse_html(html);
        assert_eq!(blocks.len(), 3);

        match &blocks[0] {
            ContentBlock::Paragraph { .. } => {}
            _ => panic!("Expected Paragraph block"),
        }

        match &blocks[1] {
            ContentBlock::Image { url, alt, link_url } => {
                assert!(url.contains("fo0hpgcvhvigtpjof6g0.png"));
                assert!(alt.as_ref().unwrap().contains("coding-agent-router"));
                assert!(link_url.is_some());
            }
            _ => panic!("Expected Image block at index 1"),
        }

        match &blocks[2] {
            ContentBlock::Paragraph { .. } => {}
            _ => panic!("Expected Paragraph block at index 2"),
        }
    }
}



