# HTML Compatibility & Normalization Contract Matrix
## `react-native-fast-html-parser`

This document defines the **formal HTML compatibility and data normalization contract** for `react-native-fast-html-parser`. It explicitly documents how every standard HTML element, custom tag, and attribute is processed, normalized, preserved, or ignored by the native parsing engine.

---

## 1. Terminology & Classification Definitions

Every HTML construct is assigned one of the following classification statuses:

| Status | Meaning | Processing Behavior |
| :--- | :--- | :--- |
| **`SUPPORTED`** | First-class AST block or inline node. | Extracted with dedicated typed fields and rendered by built-in `<FastHtmlView />`. |
| **`NORMALIZED`** | Flattened or restructured for mobile layout. | Wrapper elements (e.g. redundant `<div>` chains) are collapsed into semantic content blocks without losing text or inline children. |
| **`PRESERVED`** | Retained in generic attributes or custom nodes. | Custom tags, `data-*`, `aria-*`, and arbitrary attributes are preserved in the AST and accessible via `customRenderers`. |
| **`IGNORED`** | Discarded by design for security or safety. | Elements like `<script>`, `<style>`, and `<meta>` are discarded to prevent security vulnerabilities or styling conflicts with React Native. |
| **`UNSUPPORTED`** | Out of scope for a native content engine. | Complex browser-specific APIs (Canvas, WebGL, active HTML forms) are not evaluated; fallback to text or `customRenderers`. |

---

## 2. Document Structure & Layout Containers

| HTML Tag | Classification | Normalization & Extraction Behavior | Rendered Output |
| :--- | :---: | :--- | :--- |
| `<html>`, `<body>` | **`NORMALIZED`** | Root wrappers are unwrapped; child elements become top-level `ContentBlock[]`. | Parent container `<View>` |
| `<head>`, `<meta>`, `<title>` | **`NORMALIZED`** | `<head>` is skipped; `<title>` is extracted into `ParsedArticle.title`. | None (Metadata only) |
| `<script>`, `<style>`, `<noscript>` | **`IGNORED`** | 100% stripped for security (XSS prevention) and styling isolation. | None (Zero execution) |
| `<div>`, `<section>`, `<article>`, `<main>` | **`NORMALIZED`** | Multi-level wrapper chains are collapsed. If styled with CSS grid/table classes, mapped to tables; otherwise children are hoisted into blocks. | Native `<View>` or inner blocks |
| `<aside>`, `<nav>`, `<header>`, `<footer>` | **`NORMALIZED`** | Semantic landmark wrappers are unnested; inner text and blocks are preserved. | Direct block sequence |

---

## 3. Headings & Text Formatting

| HTML Tag | Classification | Normalization & Extraction Behavior | Rendered Output |
| :--- | :---: | :--- | :--- |
| `<h1>` – `<h6>` | **`SUPPORTED`** | Extracted as `ContentBlock::Heading { level: 1..6, children }`. Level is 1-indexed. | `<Text accessibilityRole="header" aria-level={level}>` |
| `<p>` | **`SUPPORTED`** | Extracted as `ContentBlock::Paragraph { children }`. Whitespace is normalized. | `<Text style={styles.paragraph}>` |
| `<blockquote>`, `<q>` | **`SUPPORTED`** | Extracted as `ContentBlock::Quote { children: ContentBlock[] }`. Supports nested blocks. | `<View style={styles.quoteContainer}>` |
| `<pre>`, `<code>` (Block) | **`SUPPORTED`** | Extracted as `ContentBlock::CodeBlock { code, language }`. Language is auto-detected from `class="language-xyz"`. | `<View style={styles.codeBlockContainer}>` |
| `<hr>` | **`SUPPORTED`** | Extracted as `ContentBlock::Separator`. | `<View style={styles.separator}>` |
| `<address>` | **`NORMALIZED`** | Mapped to `Paragraph` with italic styling. | `<Text style={styles.italic}>` |

---

## 4. Inline Typography & Phrasing

| HTML Tag | Classification | Normalization & Extraction Behavior | Rendered Output |
| :--- | :---: | :--- | :--- |
| `<b>`, `<strong>` | **`SUPPORTED`** | Extracted as `InlineNode::Bold { children }`. Supports recursive nesting. | `<Text style={styles.bold}>` |
| `<i>`, `<em>` | **`SUPPORTED`** | Extracted as `InlineNode::Italic { children }`. Supports recursive nesting. | `<Text style={styles.italic}>` |
| `<u>`, `<ins>` | **`SUPPORTED`** | Extracted as `InlineNode::Underline { children }`. | `<Text style={{ textDecorationLine: 'underline' }}>` |
| `<s>`, `<del>`, `<strike>` | **`SUPPORTED`** | Extracted as `InlineNode::Strikethrough { children }`. | `<Text style={{ textDecorationLine: 'line-through' }}>` |
| `<a>` | **`SUPPORTED`** | Extracted as `InlineNode::Link { url, text, children }`. Captures `href`. | `<Text accessibilityRole="link" onPress={...}>` |
| `<code>` (Inline), `<kbd>`, `<samp>` | **`SUPPORTED`** | Extracted as `InlineNode::InlineCode { text }`. | `<Text style={styles.inlineCode}>` |
| `<br>` | **`SUPPORTED`** | Extracted as `InlineNode::Break`. | `<Text>{'\n'}</Text>` |
| `<sub>`, `<sup>` | **`SUPPORTED`** | Extracted as `InlineNode::Subscript` / `InlineNode::Superscript`. | `<Text style={styles.sub/sup}>` |
| `<mark>` | **`SUPPORTED`** | Extracted as `InlineNode` with background highlight. | `<Text style={{ backgroundColor: '#fef08a' }}>` |
| `<span>` | **`NORMALIZED`** | Inline wrapper unwrapped; classes/styles captured in `InlineNode.attributes`. | Continuous nested `<Text>` |
| `<wbr>` | **`NORMALIZED`** | Zero-width break opportunity mapped to soft hyphen or empty inline. | None |

---

## 5. Lists & Hierarchies

| HTML Tag | Classification | Normalization & Extraction Behavior | Rendered Output |
| :--- | :---: | :--- | :--- |
| `<ul>` | **`SUPPORTED`** | Extracted as `ContentBlock::List { ordered: false, items }`. | Bulleted flex row `<View>` |
| `<ol>` | **`SUPPORTED`** | Extracted as `ContentBlock::List { ordered: true, items }`. | Numbered (`1.`, `2.`) flex row `<View>` |
| `<li>` | **`SUPPORTED`** | Extracted as `ListItem { children: InlineNode[], nestedBlocks: ContentBlock[] }`. | List item container |
| `<dl>`, `<dt>`, `<dd>` | **`SUPPORTED`** | Extracted as `ContentBlock::DefinitionList { items: DefinitionItem[] }`. | Definition list `<View>` with term & description |

---

## 6. Tables & 2D Data Grids

| HTML Tag | Classification | Normalization & Extraction Behavior | Rendered Output |
| :--- | :---: | :--- | :--- |
| `<table>` | **`SUPPORTED`** | Extracted as `ContentBlock::Table { rows: TableRow[] }`. | `<ScrollView horizontal>` + `<View style={styles.table}>` |
| `<thead>`, `<tbody>`, `<tfoot>` | **`NORMALIZED`** | Semantic sections grouped into chronological rows; header row marked. | Table row sequence |
| `<tr>` | **`SUPPORTED`** | Extracted as `TableRow::Row { cells: TableCell[] }`. | `<View style={styles.tableRow}>` |
| `<th>` | **`SUPPORTED`** | Extracted as `TableCell::Cell { children }` with header formatting. | `<View style={styles.tableHeaderCell}>` |
| `<td>` | **`SUPPORTED`** | Extracted as `TableCell::Cell { children: InlineNode[] }`. | `<View style={styles.tableCell}>` |
| `<div class="table|grid">` | **`SUPPORTED`** | Auto-detected CSS div-grid table; normalized into `ContentBlock::Table`. | Responsive table grid |
| `<caption>`, `<colgroup>`, `<col>` | **`NORMALIZED`** | Captions preserved in block attributes; colgroups normalized into cell styles. | Table header / cell formatting |

---

## 7. Media, Images & Embeds

| HTML Tag | Classification | Normalization & Extraction Behavior | Rendered Output |
| :--- | :---: | :--- | :--- |
| `<img>` | **`SUPPORTED`** | Extracted as `ContentBlock::Image { url, alt, width, height, attributes }`. | Native `<Image accessibilityRole="image">` or `expo-image` |
| `<figure>`, `<figcaption>` | **`SUPPORTED`** | Extracted as `ContentBlock::Figure { url, alt, caption, attributes }`. | Container `<View>` + `<Image>` + `<Text style={styles.caption}>` |
| `<picture>`, `<source>` | **`NORMALIZED`** | Selects optimal image candidate URL and captures `srcset` in attributes. | Image block |
| `<video>`, `<audio>` | **`PRESERVED`** | Extracted with `src`, `poster`, `controls`, `width`, `height`. | Accessible via `customRenderers.video` |
| `<iframe>`, `<embed>`, `<object>` | **`PRESERVED`** | Extracted with `src`, `title`, `width`, `height`, and attributes. | Accessible via `customRenderers.iframe` |

---

## 8. Forms & Interactive Elements

| HTML Tag | Classification | Normalization & Extraction Behavior | Rendered Output |
| :--- | :---: | :--- | :--- |
| `<details>`, `<summary>` | **`PRESERVED`** | Captured with title and collapsible child blocks in AST. | Accessible via `customRenderers.details` |
| `<form>`, `<input>`, `<button>` | **`NORMALIZED`** | Text content and labels preserved; active web form submissions ignored. | Rendered as text / button (use native React components) |
| `<textarea>`, `<select>`, `<option>` | **`NORMALIZED`** | Selected values and labels extracted as text content. | Rendered as text |

---

## 9. Custom Web Components & Arbitrary Tags

| Construct | Classification | Normalization & Extraction Behavior | Rendered Output |
| :--- | :---: | :--- | :--- |
| `<custom-element>`, `<app-*>` | **`PRESERVED`** | Tag name retained, inner children parsed, all attributes captured in `attributes`. | Resolved via `customRenderers['tag-name']` |

---

## 10. Attribute Extraction & Preservation

| Attribute | Classification | Normalization Rule |
| :--- | :---: | :--- |
| `id` | **`PRESERVED`** | Stored on `ContentBlock.id`. |
| `class` / `className` | **`PRESERVED`** | Stored on `ContentBlock.className` and used for syntax/grid detection. |
| `style` (Inline CSS) | **`PRESERVED`** | Stored on `ContentBlock.attributes.style`. |
| `data-*` (e.g. `data-track-id`) | **`PRESERVED`** | Stored in `ContentBlock.attributes['data-*']`. |
| `aria-*` & `role` | **`PRESERVED`** | Stored in `ContentBlock.attributes` and used for accessibility props. |
| `dir` & `lang` | **`PRESERVED`** | Stored in `attributes` and evaluated by mobile OS BiDi text engine. |
| `href`, `target`, `rel` | **`PRESERVED`** | Stored on `InlineNode.url` and `InlineNode.attributes`. |
| `src`, `alt`, `title`, `poster` | **`PRESERVED`** | Stored on `ContentBlock.url`, `ContentBlock.alt`, `ContentBlock.caption`. |
| `width`, `height` | **`PRESERVED`** | Parsed into numbers for zero-CLS aspect ratio calculations. |

---

## 11. HTML Entity Decoding Table

All standard HTML5 entities are decoded natively in compiled C++ (Lexbor) before AST generation:

| Entity Type | Examples | Decoded Output |
| :--- | :--- | :--- |
| **Named Entities** | `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;` | `&`, `<`, `>`, `"`, `'` |
| **Typographic Symbols** | `&mdash;`, `&ndash;`, `&hellip;`, `&copy;`, `&reg;` | `—`, `–`, `…`, `©`, `®` |
| **Whitespace Entities** | `&nbsp;`, `&ensp;`, `&emsp;`, `&thinsp;` | `\u00A0`, `\u2002`, `\u2003`, `\u2009` |
| **Decimal Codes** | `&#38;`, `&#160;`, `&#8212;` | `&`, `\u00A0`, `—` |
| **Hexadecimal Codes** | `&#x26;`, `&#xA0;`, `&#x2014;`, `&#x1F600;` | `&`, `\u00A0`, `—`, `😀` |
