# react-native-fast-html-parser

[![npm](https://img.shields.io/npm/v/react-native-fast-html-parser?color=orange&label=npm)](https://www.npmjs.com/package/react-native-fast-html-parser)
[![license](https://img.shields.io/npm/l/react-native-fast-html-parser?color=green&label=license)](LICENSE)
[![platforms](https://img.shields.io/badge/platforms-iOS%20%7C%20Android-lightgrey)](https://github.com/abhishekce17/react-native-fast-html-parser)
[![nitro](https://img.shields.io/badge/powered%20by-Nitro%20Modules-blue)](https://nitro.margelo.com)
[![rust](https://img.shields.io/badge/core-Rust-orange?logo=rust)](https://www.rust-lang.org)

A high-performance HTML Content & Editorial Pipeline for React Native. Powered by a compiled native Rust core engine and integrated via direct C++ JSI using Margelo Nitro Modules.

It provides dual execution modes: **Lazy native-backed AST access** that avoids materializing the complete AST in JavaScript for instant UI rendering, and a **1-Pass Native JSON pipeline** (`parseHTMLToJSON`) for offline persistence, background workers, and global state management.

---

## 📑 Table of Contents

- [Features](#-features)
- [Empirical Benchmarks](#-empirical-benchmarks)
- [Installation & Setup](#-installation--setup)
- [Architecture Overview](#-architecture-overview)
- [Core API Reference](#-core-api-reference)
  - [`parseHTML(html)`](#1-parsehtmlhtml)
  - [`parseHTMLToJSON(html)`](#2-parsehtmltojsonhtml)
- [React Native UI Components](#-react-native-ui-components)
  - [`<HtmlRenderer />`](#1-htmlrenderer-)
  - [`<VirtualizedHtmlRenderer />`](#2-virtualizedhtmlrenderer-)
- [AST Traversal & Wrapper Utilities](#-ast-traversal--wrapper-utilities)
  - [`getBlocks(article)`](#1-getblocksarticle)
  - [`getChildren(node)`](#2-getchildrennode)
  - [`getItems(block)`](#3-getitemsblock)
  - [`getNestedBlocks(item)`](#4-getnestedblocksitem)
  - [`getRows(block)`](#5-getrowsblock)
  - [`getCells(row)`](#6-getcellsrow)
  - [`getQuoteChildren(block)`](#7-getquotechildrenblock)
  - [`getDefItems(block)`](#8-getdefitemsblock)
- [Application Canonical Adapters](#-application-canonical-adapters)
  - [`createCanonicalAdapter(config)`](#createcanonicaladapterconfig)
- [Low-Level JSI HybridObject API](#-low-level-jsi-hybridobject-api)
  - [`ParsedArticle`](#parsedarticle)
  - [`ContentBlock`](#contentblock)
  - [`InlineNode`](#inlinenode)
  - [`ListItem`](#listitem)
  - [`TableRow` & `TableCell`](#tablerow--tablecell)
  - [`DefinitionItem`](#definitionitem)
- [Block Type Reference & Properties](#-block-type-reference--properties)
- [Advanced Recipes](#-advanced-recipes)
  - [Recipe 1: Drop-in `@shopify/flash-list` Virtualization](#recipe-1-drop-in-shopifyflash-list-virtualization)
  - [Recipe 2: Offline Caching with MMKV / SQLite](#recipe-2-offline-caching-with-mmkv--sqlite)
  - [Recipe 3: Custom Video & Rich Media Player](#recipe-3-custom-video--rich-media-player)
  - [Recipe 4: Custom Syntax Highlighting for Code Blocks](#recipe-4-custom-syntax-highlighting-for-code-blocks)
- [Full TypeScript Type Reference](#-full-typescript-type-reference)
- [HTML Compatibility Matrix](#-html-compatibility-matrix)
- [License](#-license)

---

## 🚀 Features

- **Blazing Fast Native Core**: Sub-millisecond HTML parsing written in Rust and compiled directly into native machine code.
- **Lazy Native-Backed AST**: Direct C++ JSI host pointers avoid materializing the entire AST tree in JavaScript upfront, preventing memory pressure and Hermes GC spikes.
- **1-Pass Native JSON Pipeline**: Direct `parseHTMLToJSON()` native serialization for SQLite, MMKV, WatermelonDB, and Redux caching.
- **Drop-in UI Components**: `<HtmlRenderer />` and `<VirtualizedHtmlRenderer />` with built-in accessibility roles (`header`, `link`, `image`), horizontal table scrolling, and full CSS-like tag styling.
- **Custom Renderers**: Override any block (`renderers`) or inline node (`inlineRenderers`) with custom React components.
- **Application Canonical Adapters**: Decouple parser AST from your domain schemas (e.g. RSS feed, CMS models) with `createCanonicalAdapter()`.
- **Zero-Dependency AST Wrappers**: 8 convenient helper functions to traverse blocks, inlines, lists, tables, quotes, and definition lists as standard JS arrays.
- **Strict Tag Normalization Contract**: Validated against 60+ HTML tags. See [HTML_COMPATIBILITY_MATRIX.md](./HTML_COMPATIBILITY_MATRIX.md).

---

## ⚡ Empirical Benchmarks

Measured on native Rust engine across 6 payload tiers (`yarn benchmark`):

| Payload Tier | Exact Size  | AST Blocks    | Parse Time     | 1-Pass JSON Time | Total Time     | Sustained Throughput |
| :----------- | :---------- | :------------ | :------------- | :--------------- | :------------- | :------------------- |
| **1 KB**     | 1.03 KB     | 8 blocks      | **0.025 ms**   | 0.002 ms         | **0.027 ms**   | 37.40 MB/s           |
| **10 KB**    | 10.84 KB    | 74 blocks     | **0.285 ms**   | 0.034 ms         | **0.319 ms**   | 33.24 MB/s           |
| **100 KB**   | 100.11 KB   | 674 blocks    | **1.794 ms**   | 0.137 ms         | **1.931 ms**   | 50.63 MB/s           |
| **500 KB**   | 500.43 KB   | 3,362 blocks  | **8.463 ms**   | 0.696 ms         | **9.159 ms**   | 53.36 MB/s           |
| **1 MB**     | 1,024.19 KB | 6,878 blocks  | **18.352 ms**  | 1.885 ms         | **20.237 ms**  | 49.42 MB/s           |
| **5 MB**     | 5,120.25 KB | 34,352 blocks | **100.316 ms** | 8.894 ms         | **109.210 ms** | 45.79 MB/s           |

Run the benchmark suite locally:

```bash
yarn benchmark
```

---

## 📦 Installation & Setup

```bash
# Using npm
npm install react-native-fast-html-parser react-native-nitro-modules

# Using yarn
yarn add react-native-fast-html-parser react-native-nitro-modules
```

### iOS Setup

```bash
cd ios && pod install
```

### Android Setup

No additional configuration required. Android builds automatically link native C++ and Rust JNI artifacts via Nitro Modules.

### Rebuild Application

```bash
npx react-native run-ios
# or
npx react-native run-android
```

---

## 🏗 Architecture Overview

```text
               Raw HTML Input String
                         │
                         ▼
        ┌─────────────────────────────────┐
        │       Compiled Rust Core        │
        │   Tokenization & Tag Normalizer  │
        └────────────────┬────────────────┘
                         │
         ┌───────────────┴───────────────┐
         ▼                               ▼
┌──────────────────┐           ┌──────────────────┐
│  Lazy JSI Hybrid │           │   1-Pass Native  │
│   Object Tree    │           │   JSON String    │
└────────┬─────────┘           └────────┬─────────┘
         │                              │
         ▼                              ▼
┌──────────────────┐           ┌──────────────────┐
│  <HtmlRenderer>  │           │   MMKV / SQLite  │
│  Virtual List UI │           │  State & Cache   │
└──────────────────┘           └──────────────────┘
```

---

## 🔌 Core API Reference

### 1. `parseHTML(html)`

Parses raw HTML into a lazy native JSI `ParsedArticle` HybridObject. Elements are traversed on-demand via direct C++ pointers with zero upfront JavaScript heap allocations.

```typescript
import { parseHTML, type ParsedArticle } from 'react-native-fast-html-parser';

const html = `
  <article>
    <h1>Supercharged React Native</h1>
    <p>Render <b>bold</b> text with sub-millisecond parsing speed.</p>
  </article>
`;

const article: ParsedArticle | null = parseHTML(html);

if (article) {
  console.log('Total Block Count:', article.length);
  const firstBlock = article.getBlock(0);
  console.log('First Block Type:', firstBlock?.type); // "Heading"
}
```

---

### 2. `parseHTMLToJSON(html)`

Parses raw HTML and directly serializes it to a compact JSON string in a single native pass inside Rust. The native memory is immediately released. Ideal for background workers, offline persistence, and Redux/Zustand state slices.

```typescript
import {
  parseHTMLToJSON,
  type ParsedArticleData,
} from 'react-native-fast-html-parser';

const html = `<h2>Fast Pipeline</h2><p>Saved directly to storage.</p>`;

// 1. Get raw JSON string from native Rust pipeline
const jsonString: string = parseHTMLToJSON(html);

// 2. Parse into typed JS object tree if needed
const data: ParsedArticleData = JSON.parse(jsonString);

console.log('Blocks:', data.blocks.length);
console.log('First Block Type:', data.blocks[0].type); // "heading"
```

---

## 🎨 React Native UI Components

### 1. `<HtmlRenderer />`

The standard drop-in component to render HTML into native React Native views with rich styling, link handlers, and custom renderer overrides.

#### Props

| Prop              | Type                                     | Default           | Description                                                                |
| :---------------- | :--------------------------------------- | :---------------- | :------------------------------------------------------------------------- |
| `html`            | `string`                                 | `undefined`       | The raw HTML string to parse and render.                                   |
| `parsedAst`       | `ParsedArticle \| null`                  | `undefined`       | Pre-parsed AST object (used if parsed ahead of time).                      |
| `baseStyle`       | `TextStyle`                              | `undefined`       | Base text style inherited by all inline text elements.                     |
| `tagsStyles`      | `Record<string, TextStyle \| ViewStyle>` | `{}`              | Style overrides keyed by tag name (`h1`, `p`, `a`, `code`, `table`, etc.). |
| `renderers`       | `Record<string, CustomBlockRenderer>`    | `{}`              | Custom React component overrides for block types.                          |
| `inlineRenderers` | `Record<string, CustomInlineRenderer>`   | `{}`              | Custom React component overrides for inline formatting.                    |
| `onLinkPress`     | `(url: string) => void`                  | `Linking.openURL` | Callback triggered when an `<a>` anchor link is pressed.                   |
| `style`           | `ViewStyle`                              | `undefined`       | Container style for the root `<View>`.                                     |

#### Complete Example: Styling & Custom Renderers

```tsx
import React from 'react';
import { ScrollView, View, Text, StyleSheet, Alert } from 'react-native';
import {
  HtmlRenderer,
  type CustomBlockRenderer,
  type CustomInlineRenderer,
} from 'react-native-fast-html-parser';

// Custom renderer for Code blocks
const CustomCodeRenderer: CustomBlockRenderer = ({ block }) => {
  return (
    <View style={styles.codeContainer}>
      <Text style={styles.codeLang}>{block.language || 'code'}</Text>
      <Text style={styles.codeText}>{block.code}</Text>
    </View>
  );
};

// Custom renderer for inline bold text
const CustomBoldRenderer: CustomInlineRenderer = ({ node, defaultRender }) => {
  return <Text style={styles.customBold}>{node.text}</Text>;
};

export function ArticleDetailScreen() {
  const html = `
    <h1>Deep Dive into Native Modules</h1>
    <p>Learn how to integrate <b>Rust</b> with direct <i>JSI</i> bindings.</p>
    <pre><code class="language-typescript">const fast = parseHTML(raw);</code></pre>
    <p>Read the documentation <a href="https://github.com">here</a>.</p>
  `;

  return (
    <ScrollView style={styles.screen}>
      <HtmlRenderer
        html={html}
        baseStyle={{ fontSize: 16, color: '#334155', lineHeight: 24 }}
        tagsStyles={{
          h1: {
            fontSize: 26,
            color: '#0f172a',
            fontWeight: '800',
            marginBottom: 12,
          },
          a: { color: '#2563eb', textDecorationLine: 'underline' },
        }}
        renderers={{
          CodeBlock: CustomCodeRenderer,
        }}
        inlineRenderers={{
          Bold: CustomBoldRenderer,
        }}
        onLinkPress={(url) => {
          Alert.alert('Link Clicked', url);
        }}
      />
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  screen: { flex: 1, backgroundColor: '#ffffff', padding: 16 },
  codeContainer: {
    backgroundColor: '#1e293b',
    padding: 12,
    borderRadius: 8,
    marginVertical: 8,
  },
  codeLang: {
    color: '#94a3b8',
    fontSize: 12,
    fontWeight: '700',
    textTransform: 'uppercase',
  },
  codeText: { color: '#f8fafc', fontFamily: 'Courier', marginTop: 4 },
  customBold: { fontWeight: '900', color: '#09090b' },
});
```

---

### `parsedAst` — Parse Once, Render Many

Parse the AST exactly once (e.g. in a hook or screen-level `useMemo`) and pass the same `ParsedArticle` object to any number of components. Avoids redundant native parse calls across re-renders, navigations, or list rows.

```tsx
import React, { useMemo } from 'react';
import { ScrollView, View } from 'react-native';
import {
  parseHTML,
  HtmlRenderer,
  VirtualizedHtmlRenderer,
  type ParsedArticle,
} from 'react-native-fast-html-parser';

function useArticleAst(rawHtml: string): ParsedArticle | null {
  // Parse once per `rawHtml` string — memoized across re-renders
  return useMemo(() => parseHTML(rawHtml), [rawHtml]);
}

export function ArticleScreen({ rawHtml }: { rawHtml: string }) {
  const parsedAst = useArticleAst(rawHtml);

  return (
    <ScrollView>
      {/* Preview card — same AST, no second parse */}
      <HtmlRenderer
        parsedAst={parsedAst}
        baseStyle={{ fontSize: 14, color: '#64748b' }}
        tagsStyles={{ h1: { display: 'none' } }} // hide title in preview
      />

      {/* Full article — same AST, still no second parse */}
      <VirtualizedHtmlRenderer
        parsedAst={parsedAst}
        baseStyle={{ fontSize: 16, color: '#0f172a', lineHeight: 26 }}
      />
    </ScrollView>
  );
}
```

> **Rule of thumb**: use `html` prop when the component owns the data lifecycle. Use `parsedAst` when you parse outside the component (route loader, React Query, background task) and want rendering to be pure.

---

### 2. `<VirtualizedHtmlRenderer />`

Designed for long editorial articles, documentation pages, and news feeds. Backed by React Native's `FlatList` with row recycling to sustain smooth 120 FPS scrolling regardless of document length.

#### Props

Inherits all props from `<HtmlRenderer />` plus:

| Prop                    | Type                                    | Default     | Description                                                                   |
| :---------------------- | :-------------------------------------- | :---------- | :---------------------------------------------------------------------------- |
| `ListHeaderComponent`   | `ComponentType \| ReactElement \| null` | `null`      | Header component rendered above the article (e.g. Hero image, author info).   |
| `ListFooterComponent`   | `ComponentType \| ReactElement \| null` | `null`      | Footer component rendered below the article (e.g. Comments, related stories). |
| `contentContainerStyle` | `ViewStyle`                             | `undefined` | Style applied to the `FlatList` scroll content container.                     |

#### Complete Example: Virtualized Long Document

```tsx
import React from 'react';
import { View, Text, Image, StyleSheet } from 'react-native';
import {
  VirtualizedHtmlRenderer,
  parseHTML,
} from 'react-native-fast-html-parser';

export function VirtualizedArticleScreen({ rawHtml }: { rawHtml: string }) {
  // Optional: Pre-parse AST ahead of render
  const parsedAst = React.useMemo(() => parseHTML(rawHtml), [rawHtml]);

  return (
    <VirtualizedHtmlRenderer
      parsedAst={parsedAst}
      baseStyle={{ fontSize: 16, color: '#1e293b', lineHeight: 26 }}
      tagsStyles={{
        h1: { fontSize: 28, color: '#0f172a', fontWeight: 'bold' },
        h2: { fontSize: 22, color: '#1e293b', fontWeight: '600' },
      }}
      ListHeaderComponent={
        <View style={styles.header}>
          <Image
            source={{ uri: 'https://picsum.photos/800/400' }}
            style={styles.heroImage}
          />
          <Text style={styles.headline}>Breaking Tech Editorial</Text>
          <Text style={styles.meta}>By Engineering Team • Published Today</Text>
        </View>
      }
      ListFooterComponent={
        <View style={styles.footer}>
          <Text style={styles.footerText}>
            © 2026 Editorial Group. All rights reserved.
          </Text>
        </View>
      }
      contentContainerStyle={styles.listContent}
    />
  );
}

const styles = StyleSheet.create({
  listContent: { paddingHorizontal: 16, paddingBottom: 40 },
  header: { marginBottom: 16 },
  heroImage: { width: '100%', height: 200, borderRadius: 12, marginBottom: 12 },
  headline: { fontSize: 28, fontWeight: 'bold', color: '#0f172a' },
  meta: { fontSize: 13, color: '#64748b', marginTop: 4 },
  footer: {
    marginTop: 32,
    borderTopWidth: 1,
    borderTopColor: '#e2e8f0',
    paddingTop: 16,
  },
  footerText: { fontSize: 13, color: '#94a3b8', textAlign: 'center' },
});
```

---

## 🛠 AST Traversal & Wrapper Utilities

The library exports 8 ergonomic traversal helper functions in `react-native-fast-html-parser` that convert low-level JSI HostObject getters into standard JavaScript arrays:

### 1. `getBlocks(article)`

Extracts all top-level `ContentBlock[]` elements from a `ParsedArticle`.

```typescript
import {
  parseHTML,
  getBlocks,
  type ContentBlock,
} from 'react-native-fast-html-parser';

const article = parseHTML(
  '<h1>Title</h1><p>First paragraph.</p><p>Second paragraph.</p>'
);
const blocks: ContentBlock[] = getBlocks(article);

console.log(`Extracted ${blocks.length} blocks`);
blocks.forEach((block, index) => {
  console.log(`Block #${index}: ${block.type}`);
});
```

---

### 2. `getChildren(node)`

Extracts all inline formatted child nodes (`InlineNode[]`) from a `ContentBlock`, `InlineNode`, `TableCell`, or `ListItem`.

```typescript
import {
  parseHTML,
  getBlocks,
  getChildren,
  type InlineNode,
} from 'react-native-fast-html-parser';

const article = parseHTML(
  '<p>Welcome to <b>React Native</b> with <a href="https://nitro.margelo.com">Nitro Modules</a>.</p>'
);
const paragraph = getBlocks(article)[0];

const inlines: InlineNode[] = getChildren(paragraph);
inlines.forEach((inline) => {
  console.log(
    `Type: ${inline.type}, Text: "${inline.text}", URL: "${inline.url}"`
  );
});
```

---

### 3. `getItems(block)`

Extracts all `ListItem[]` entries from an ordered (`<ol>`) or unordered (`<ul>`) `List` block.

```typescript
import {
  parseHTML,
  getBlocks,
  getItems,
  getChildren,
  type ListItem,
} from 'react-native-fast-html-parser';

const article = parseHTML(`
  <ul>
    <li>First item</li>
    <li>Second item with <b>bold</b> text</li>
  </ul>
`);
const listBlock = getBlocks(article)[0];

if (listBlock.type === 'List') {
  const items: ListItem[] = getItems(listBlock);
  items.forEach((item, index) => {
    const textNodes = getChildren(item);
    console.log(`Item ${index + 1}:`, textNodes.map((n) => n.text).join(''));
  });
}
```

---

### 4. `getNestedBlocks(item)`

Extracts nested sub-blocks (`ContentBlock[]`) contained inside a `ListItem` (for multi-level hierarchical lists).

```typescript
import {
  parseHTML,
  getBlocks,
  getItems,
  getNestedBlocks,
  type ContentBlock,
} from 'react-native-fast-html-parser';

const article = parseHTML(`
  <ul>
    <li>
      Parent item
      <ul>
        <li>Sub item 1</li>
        <li>Sub item 2</li>
      </ul>
    </li>
  </ul>
`);
const listBlock = getBlocks(article)[0];
const parentItem = getItems(listBlock)[0];

const nestedBlocks: ContentBlock[] = getNestedBlocks(parentItem);
console.log('Nested sub-lists count:', nestedBlocks.length);
```

---

### 5. `getRows(block)`

Extracts all `TableRow[]` entries from a `Table` block.

```typescript
import {
  parseHTML,
  getBlocks,
  getRows,
  type TableRow,
} from 'react-native-fast-html-parser';

const article = parseHTML(`
  <table>
    <tr><th>Framework</th><th>Language</th></tr>
    <tr><td>React Native</td><td>TypeScript</td></tr>
    <tr><td>Nitro Modules</td><td>C++ / Rust</td></tr>
  </table>
`);
const tableBlock = getBlocks(article)[0];

const rows: TableRow[] = getRows(tableBlock);
console.log(`Table has ${rows.length} rows`);
```

---

### 6. `getCells(row)`

Extracts all `TableCell[]` entries from a `TableRow`.

```typescript
import {
  parseHTML,
  getBlocks,
  getRows,
  getCells,
  getChildren,
  type TableCell,
} from 'react-native-fast-html-parser';

const article = parseHTML(
  '<table><tr><td>Col 1</td><td>Col 2</td><td>Col 3</td></tr></table>'
);
const tableBlock = getBlocks(article)[0];
const firstRow = getRows(tableBlock)[0];

const cells: TableCell[] = getCells(firstRow);
cells.forEach((cell, index) => {
  const cellInlines = getChildren(cell);
  console.log(`Cell ${index}:`, cellInlines.map((c) => c.text).join(''));
});
```

---

### 7. `getQuoteChildren(block)`

Extracts nested `ContentBlock[]` elements from a `Quote` (`<blockquote>`) block.

```typescript
import {
  parseHTML,
  getBlocks,
  getQuoteChildren,
  getChildren,
  type ContentBlock,
} from 'react-native-fast-html-parser';

const article = parseHTML(`
  <blockquote>
    <p>Simplicity is prerequisite for reliability.</p>
    <p>— Edsger W. Dijkstra</p>
  </blockquote>
`);
const quoteBlock = getBlocks(article)[0];

const subBlocks: ContentBlock[] = getQuoteChildren(quoteBlock);
subBlocks.forEach((child) => {
  if (child.type === 'Paragraph') {
    console.log(
      'Quote line:',
      getChildren(child)
        .map((c) => c.text)
        .join('')
    );
  }
});
```

---

### 8. `getDefItems(block)`

Extracts all `DefinitionItem[]` entries from a `DefinitionList` (`<dl>`) block.

```typescript
import {
  parseHTML,
  getBlocks,
  getDefItems,
  type DefinitionItem,
} from 'react-native-fast-html-parser';

const article = parseHTML(`
  <dl>
    <dt>JSI</dt>
    <dd>JavaScript Interface for direct C++ to JavaScript communication.</dd>
    <dt>Nitro Modules</dt>
    <dd>Next-generation React Native native module framework.</dd>
  </dl>
`);
const dlBlock = getBlocks(article)[0];

const defItems: DefinitionItem[] = getDefItems(dlBlock);
defItems.forEach((item) => {
  const term = item.getTerm(0)?.text;
  const def = item.getDef(0)?.text;
  console.log(`${term} => ${def}`);
});
```

---

## 🔀 Application Canonical Adapters

### `createCanonicalAdapter(config)`

Decouple parser AST internals from your proprietary application schema (such as NewsFeed, Blog, or CMS models) using `createCanonicalAdapter()`.

```typescript
import {
  createCanonicalAdapter,
  parseHTMLToJSON,
  type ParsedArticleData,
  type ContentBlockData,
} from 'react-native-fast-html-parser';

// 1. Define your application's domain schema
interface AppDomainBlock {
  blockId: string;
  kind: 'header' | 'text' | 'media' | 'raw';
  payload: any;
}

interface AppDomainArticle {
  id: string;
  headline: string;
  content: AppDomainBlock[];
  wordCount: number;
}

// 2. Create the canonical adapter
const appAdapter = createCanonicalAdapter<AppDomainArticle, AppDomainBlock>({
  transformers: {
    heading: (block, index) => ({
      blockId: `heading-${index}`,
      kind: 'header',
      payload: {
        level: block.level,
        text: block.children.map((c) => c.text).join(''),
      },
    }),
    paragraph: (block, index) => ({
      blockId: `p-${index}`,
      kind: 'text',
      payload: { text: block.children.map((c) => c.text).join('') },
    }),
    image: (block, index) => ({
      blockId: `img-${index}`,
      kind: 'media',
      payload: { sourceUrl: block.url, caption: block.caption },
    }),
  },
  transformBlock: (block, index) => ({
    blockId: `block-${index}`,
    kind: 'raw',
    payload: block,
  }),
  transformArticle: (article, domainBlocks) => ({
    id: `art-${Date.now()}`,
    headline: article.title || 'Untitled',
    content: domainBlocks,
    wordCount: domainBlocks.length * 25,
  }),
});

// 3. Adapt parsed JSON data directly to your schema
const rawJson = parseHTMLToJSON('<h1>Title</h1><p>Body text.</p>');
const parsedJson: ParsedArticleData = JSON.parse(rawJson);

const canonicalDocument: AppDomainArticle = appAdapter.adapt(parsedJson);
console.log('Canonical Doc:', canonicalDocument);
```

---

## ⚙️ Low-Level JSI HybridObject API

When using `parseHTML()`, you interact directly with high-performance C++ JSI `HybridObject` instances:

### `ParsedArticle`

| Property / Method         | Return Type            | Description                                          |
| :------------------------ | :--------------------- | :--------------------------------------------------- |
| `length`                  | `number`               | Total number of top-level content blocks.            |
| `getBlock(index: number)` | `ContentBlock \| null` | Returns the `ContentBlock` at index without copying. |
| `toJSON()`                | `string`               | Serializes the article into a JSON string natively.  |

```typescript
const article = parseHTML('<h1>Title</h1><p>Body</p>');
if (article) {
  const count = article.length; // 2
  const block = article.getBlock(0); // ContentBlock { type: 'Heading', level: 1 }
  const json = article.toJSON(); // compact JSON string
}
```

#### `article.toJSON()` vs `parseHTMLToJSON()` — when to use each

|                   | `article.toJSON()`                                                                     | `parseHTMLToJSON(html)`                   |
| :---------------- | :------------------------------------------------------------------------------------- | :---------------------------------------- |
| **When**          | You already have a `ParsedArticle` (e.g. used for rendering first, then want to cache) | You only need JSON — no live rendering    |
| **Native memory** | Freed when `article` is GC'd                                                           | Freed immediately after the call          |
| **JSI overhead**  | 1 extra JSI call on an existing HybridObject                                           | 0 — native→JSON in a single Rust pass     |
| **Best for**      | Cache-after-render, debug `console.log`                                                | Background workers, MMKV/SQLite pipelines |

```typescript
import { parseHTML, parseHTMLToJSON } from 'react-native-fast-html-parser';

const html = '<h1>Architecture</h1><p>JSI is fast.</p>';

// Path A — render first, then snapshot to JSON
const article = parseHTML(html);
if (article) {
  // ... pass to <HtmlRenderer parsedAst={article} />
  const json = article.toJSON(); // serialize after the fact
  await AsyncStorage.setItem('cache', json);
}

// Path B — JSON only, never materialise the HybridObject tree
const json = parseHTMLToJSON(html); // 1-pass Rust → JSON
await AsyncStorage.setItem('cache', json);
```

---

### `ContentBlock`

| Field / Method     | Return Type              | Applicable Block Types              | Description                                                                                                                                                   |
| :----------------- | :----------------------- | :---------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `type`             | `string`                 | All                                 | Block type name (`Paragraph`, `Heading`, `List`, `Table`, `Image`, `Figure`, `CodeBlock`, `Quote`, `DefinitionList`, `Video`, `Audio`, `Embed`, `Separator`). |
| `level`            | `number`                 | `Heading`                           | Heading level (1 to 6).                                                                                                                                       |
| `url`              | `string`                 | `Image`, `Figure`                   | Image asset URL.                                                                                                                                              |
| `alt`              | `string`                 | `Image`, `Figure`                   | Accessibility alternative text.                                                                                                                               |
| `caption`          | `string`                 | `Figure`, `Video`, `Audio`, `Embed` | Caption or subtitle text.                                                                                                                                     |
| `code`             | `string`                 | `CodeBlock`                         | Raw source code text.                                                                                                                                         |
| `language`         | `string`                 | `CodeBlock`                         | Syntax language (e.g. `typescript`, `rust`, `python`).                                                                                                        |
| `src`              | `string`                 | `Video`, `Audio`, `Embed`           | Media source URL or embed iframe target.                                                                                                                      |
| `poster`           | `string`                 | `Video`                             | Video preview thumbnail poster URL.                                                                                                                           |
| `title`            | `string`                 | `Embed`                             | Title attribute of embed/iframe.                                                                                                                              |
| `ordered`          | `boolean`                | `List`                              | `true` for `<ol>`, `false` for `<ul>`.                                                                                                                        |
| `childCount`       | `number`                 | `Paragraph`, `Heading`, `Quote`     | Number of inline child nodes or nested blocks.                                                                                                                |
| `getChild(i)`      | `InlineNode \| null`     | `Paragraph`, `Heading`              | Returns the inline child node at index `i`.                                                                                                                   |
| `quoteChildCount`  | `number`                 | `Quote`                             | Number of child blocks inside a blockquote.                                                                                                                   |
| `getQuoteChild(i)` | `ContentBlock \| null`   | `Quote`                             | Returns the nested `ContentBlock` at index `i`.                                                                                                               |
| `itemCount`        | `number`                 | `List`, `DefinitionList`            | Number of list items or definition items.                                                                                                                     |
| `getItem(i)`       | `ListItem \| null`       | `List`                              | Returns the `ListItem` at index `i`.                                                                                                                          |
| `rowCount`         | `number`                 | `Table`                             | Number of rows in table.                                                                                                                                      |
| `getRow(i)`        | `TableRow \| null`       | `Table`                             | Returns the `TableRow` at index `i`.                                                                                                                          |
| `defItemCount`     | `number`                 | `DefinitionList`                    | Number of term/definition pairs.                                                                                                                              |
| `getDefItem(i)`    | `DefinitionItem \| null` | `DefinitionList`                    | Returns the `DefinitionItem` at index `i`.                                                                                                                    |

---

### `InlineNode`

| Field / Method | Return Type          | Description                                                                     |
| :------------- | :------------------- | :------------------------------------------------------------------------------ |
| `type`         | `string`             | Node type: `'Text'`, `'Bold'`, `'Italic'`, `'Link'`, `'InlineCode'`, `'Break'`. |
| `text`         | `string`             | Text content of the node.                                                       |
| `url`          | `string`             | Destination URL (for `Link` nodes).                                             |
| `childCount`   | `number`             | Number of nested inline formatting nodes.                                       |
| `getChild(i)`  | `InlineNode \| null` | Returns the nested inline node at index `i`.                                    |

```typescript
import {
  parseHTML,
  getBlocks,
  getChildren,
} from 'react-native-fast-html-parser';

const article = parseHTML(
  '<p>Visit <a href="https://nitro.margelo.com"><b>Nitro Modules</b></a> now.</p>'
);
const para = getBlocks(article)[0];

// Raw JSI traversal without wrapper helpers
for (let i = 0; i < para.childCount; i++) {
  const node = para.getChild(i);
  if (!node) continue;

  console.log('type:', node.type); // 'Text' | 'Bold' | 'Italic' | 'Link' | 'InlineCode' | 'Break'
  console.log('text:', node.text); // plain text content
  console.log('url:', node.url); // non-empty only for Link nodes

  // Drill into nested formatting (e.g. <a><b>text</b></a>)
  for (let j = 0; j < node.childCount; j++) {
    const nested = node.getChild(j);
    console.log('  nested:', nested?.type, nested?.text);
  }
}
```

---

### `ListItem`

| Field / Method | Return Type            | Description                                     |
| :------------- | :--------------------- | :---------------------------------------------- |
| `childCount`   | `number`               | Number of inline child formatting nodes.        |
| `getChild(i)`  | `InlineNode \| null`   | Returns the inline child node at index `i`.     |
| `nestedCount`  | `number`               | Number of nested sub-lists or sub-blocks.       |
| `getNested(i)` | `ContentBlock \| null` | Returns the nested `ContentBlock` at index `i`. |

```typescript
import { parseHTML, getBlocks, getItems } from 'react-native-fast-html-parser';

const article = parseHTML(`
  <ul>
    <li>Plain text item</li>
    <li>Item with <b>bold</b> inline
      <ul><li>Sub-item</li></ul>
    </li>
  </ul>
`);
const listBlock = getBlocks(article)[0];

// Raw JSI traversal without wrapper helpers
for (let i = 0; i < listBlock.itemCount; i++) {
  const item = listBlock.getItem(i);
  if (!item) continue;

  // Inline text inside the <li>
  for (let c = 0; c < item.childCount; c++) {
    const inline = item.getChild(c);
    console.log('  inline:', inline?.type, inline?.text);
  }

  // Nested lists / sub-blocks inside the <li>
  for (let n = 0; n < item.nestedCount; n++) {
    const nested = item.getNested(n);
    console.log('  nested block type:', nested?.type); // 'List'
  }
}
```

---

### `TableRow` & `TableCell`

| Structure       | Method                                | Description                               |
| :-------------- | :------------------------------------ | :---------------------------------------- |
| **`TableRow`**  | `cellCount: number`                   | Total number of cells in the row.         |
|                 | `getCell(index): TableCell \| null`   | Returns the `TableCell` at index.         |
| **`TableCell`** | `childCount: number`                  | Total number of inline nodes in the cell. |
|                 | `getChild(index): InlineNode \| null` | Returns the `InlineNode` at index.        |

```typescript
import { parseHTML, getBlocks } from 'react-native-fast-html-parser';

const article = parseHTML(`
  <table>
    <tr><th>Name</th><th>Version</th></tr>
    <tr><td>Nitro</td><td>0.20</td></tr>
  </table>
`);
const tableBlock = getBlocks(article)[0]; // ContentBlock { type: 'Table' }

// Raw JSI traversal without wrapper helpers
for (let r = 0; r < tableBlock.rowCount; r++) {
  const row = tableBlock.getRow(r); // TableRow
  if (!row) continue;
  const rowData: string[] = [];

  for (let c = 0; c < row.cellCount; c++) {
    const cell = row.getCell(c); // TableCell
    if (!cell) continue;
    const texts: string[] = [];

    for (let n = 0; n < cell.childCount; n++) {
      const inline = cell.getChild(n); // InlineNode
      if (inline?.text) texts.push(inline.text);
    }
    rowData.push(texts.join(''));
  }
  console.log('Row:', rowData.join(' | ')); // e.g. "Name | Version"
}
```

---

### `DefinitionItem`

| Field / Method | Return Type          | Description                                  |
| :------------- | :------------------- | :------------------------------------------- |
| `termCount`    | `number`             | Number of term (`<dt>`) inline nodes.        |
| `getTerm(i)`   | `InlineNode \| null` | Returns the `<dt>` inline node at index `i`. |
| `defCount`     | `number`             | Number of definition (`<dd>`) inline nodes.  |
| `getDef(i)`    | `InlineNode \| null` | Returns the `<dd>` inline node at index `i`. |

```typescript
import { parseHTML, getBlocks } from 'react-native-fast-html-parser';

const article = parseHTML(`
  <dl>
    <dt>JSI</dt>
    <dd>JavaScript Interface — direct C++ to JS bridge.</dd>
    <dt>Nitro</dt>
    <dd>Next-gen React Native native module framework.</dd>
  </dl>
`);
const dl = getBlocks(article)[0]; // ContentBlock { type: 'DefinitionList' }

// Raw JSI traversal without wrapper helpers
for (let i = 0; i < dl.defItemCount; i++) {
  const item = dl.getDefItem(i); // DefinitionItem
  if (!item) continue;

  // Collect term (<dt>) text
  const termParts: string[] = [];
  for (let t = 0; t < item.termCount; t++) {
    const node = item.getTerm(t);
    if (node?.text) termParts.push(node.text);
  }

  // Collect definition (<dd>) text
  const defParts: string[] = [];
  for (let d = 0; d < item.defCount; d++) {
    const node = item.getDef(d);
    if (node?.text) defParts.push(node.text);
  }

  console.log(`${termParts.join('')}: ${defParts.join('')}`);
  // "JSI: JavaScript Interface — direct C++ to JS bridge."
}
```

---

## 📖 Block Type Reference & Properties

| `block.type`         | Applicable Properties                      | Example HTML Tag Origin                                 |
| :------------------- | :----------------------------------------- | :------------------------------------------------------ |
| **`Paragraph`**      | `childCount`, `getChild(i)`                | `<p>`, `<address>`, `<div>` with inline text            |
| **`Heading`**        | `level` (1-6), `childCount`, `getChild(i)` | `<h1>`, `<h2>`, `<h3>`, `<h4>`, `<h5>`, `<h6>`          |
| **`List`**           | `ordered`, `itemCount`, `getItem(i)`       | `<ul>`, `<ol>`, `<li>`                                  |
| **`Table`**          | `rowCount`, `getRow(i)`                    | `<table>`, `<thead>`, `<tbody>`, `<tr>`, `<th>`, `<td>` |
| **`Image`**          | `url`, `alt`, `caption`                    | `<img>`, `<picture>`                                    |
| **`Figure`**         | `url`, `alt`, `caption`                    | `<figure>`, `<figcaption>`                              |
| **`CodeBlock`**      | `code`, `language`                         | `<pre><code>`                                           |
| **`Quote`**          | `quoteChildCount`, `getQuoteChild(i)`      | `<blockquote>`, `<q>`                                   |
| **`DefinitionList`** | `defItemCount`, `getDefItem(i)`            | `<dl>`, `<dt>`, `<dd>`                                  |
| **`Video`**          | `src`, `poster`, `caption`                 | `<video>`, `<source>`                                   |
| **`Audio`**          | `src`, `caption`                           | `<audio>`                                               |
| **`Embed`**          | `src`, `title`, `caption`                  | `<iframe>`, `<embed>`                                   |
| **`Separator`**      | _None_                                     | `<hr>`                                                  |

---

## 🎨 Custom `inlineRenderers` — All 6 Inline Types

Override any inline formatting node with a custom React component. The 6 inline types are: `Text`, `Bold`, `Italic`, `Link`, `InlineCode`, `Break`.

```tsx
import React from 'react';
import { Text, Pressable, Linking, StyleSheet, Alert } from 'react-native';
import {
  HtmlRenderer,
  type CustomInlineRenderer,
} from 'react-native-fast-html-parser';

// 1. Text — plain text leaf node
const CustomText: CustomInlineRenderer = ({ node }) => (
  <Text style={styles.body}>{node.text}</Text>
);

// 2. Bold — <b> / <strong>
const CustomBold: CustomInlineRenderer = ({ node }) => (
  <Text style={styles.bold}>{node.text}</Text>
);

// 3. Italic — <i> / <em>
const CustomItalic: CustomInlineRenderer = ({ node }) => (
  <Text style={styles.italic}>{node.text}</Text>
);

// 4. Link — <a href="...">  (node.url holds the destination)
const CustomLink: CustomInlineRenderer = ({ node }) => (
  <Text
    style={styles.link}
    onPress={() => Alert.alert('Navigate', node.url)}
    accessibilityRole="link"
    accessibilityHint={node.url}
  >
    {node.text}
  </Text>
);

// 5. InlineCode — <code> inside a paragraph
const CustomInlineCode: CustomInlineRenderer = ({ node }) => (
  <Text style={styles.inlineCode}>{node.text}</Text>
);

// 6. Break — <br />
const CustomBreak: CustomInlineRenderer = () => <Text>{'\n\n'}</Text>;

export function StyledArticle({ html }: { html: string }) {
  return (
    <HtmlRenderer
      html={html}
      inlineRenderers={{
        Text: CustomText,
        Bold: CustomBold,
        Italic: CustomItalic,
        Link: CustomLink,
        InlineCode: CustomInlineCode,
        Break: CustomBreak,
      }}
    />
  );
}

const styles = StyleSheet.create({
  body: { fontSize: 16, color: '#334155', lineHeight: 24 },
  bold: { fontWeight: '800', color: '#0f172a' },
  italic: { fontStyle: 'italic', color: '#475569' },
  link: {
    color: '#2563eb',
    textDecorationLine: 'underline',
    fontWeight: '600',
  },
  inlineCode: {
    fontFamily: 'Courier',
    backgroundColor: '#f1f5f9',
    color: '#dc2626',
    fontSize: 13,
    paddingHorizontal: 3,
    borderRadius: 3,
  },
});
```

> Each custom inline renderer receives `{ node, defaultRender, baseStyle }`. Call `defaultRender()` inside your component if you only want to wrap the default output (e.g. add a `Pressable` around it) rather than replace it entirely.

---

## 📐 `BlockTransformer` — Typed Block Transformer Functions

When building adapters, you can type individual block transformers using the exported `BlockTransformer<TOutput>` generic:

```typescript
import {
  type BlockTransformer,
  type HeadingBlockData,
  type ParagraphBlockData,
  type ImageBlockData,
  createCanonicalAdapter,
  parseHTMLToJSON,
  type ParsedArticleData,
} from 'react-native-fast-html-parser';

// Typed transformer for heading blocks
const headingTransformer: BlockTransformer<{ tag: string; content: string }> = (
  block,
  index
) => ({
  tag: `h${(block as HeadingBlockData).level}`,
  content:
    (block as HeadingBlockData).children?.map((c) => c.text).join('') ?? '',
});

// Typed transformer for paragraph blocks
const paragraphTransformer: BlockTransformer<{
  tag: string;
  content: string;
}> = (block, _index) => ({
  tag: 'p',
  content:
    (block as ParagraphBlockData).children?.map((c) => c.text).join('') ?? '',
});

// Typed transformer for image blocks
const imageTransformer: BlockTransformer<{
  tag: string;
  src: string;
  alt: string;
}> = (block, _index) => ({
  tag: 'img',
  src: (block as ImageBlockData).url,
  alt: (block as ImageBlockData).alt ?? '',
});

// Compose into a canonical adapter
const adapter = createCanonicalAdapter({
  transformers: {
    heading: headingTransformer,
    paragraph: paragraphTransformer,
    image: imageTransformer,
  },
  // Fallback: pass unknown blocks through unchanged
  transformBlock: (block) => block,
});

const json: ParsedArticleData = JSON.parse(
  parseHTMLToJSON('<h1>Hello</h1><p>World</p><img src="x.jpg" alt="X">')
);
console.log(adapter.adapt(json));
// { blocks: [ { tag: 'h1', content: 'Hello' }, { tag: 'p', content: 'World' }, { tag: 'img', src: 'x.jpg', alt: 'X' } ] }
```

---

## 🚀 Advanced Recipes

### Recipe 1: Drop-in `@shopify/flash-list` Virtualization

For ultra-high performance on low-end Android devices and 120Hz displays, combine `getBlocks` with `@shopify/flash-list`:

```tsx
import React, { useMemo } from 'react';
import { FlashList } from '@shopify/flash-list';
import {
  parseHTML,
  getBlocks,
  HtmlRenderer,
  type ContentBlock,
} from 'react-native-fast-html-parser';

export function FlashListArticle({ html }: { html: string }) {
  const article = useMemo(() => parseHTML(html), [html]);
  const blocks = useMemo(() => getBlocks(article), [article]);

  const renderItem = ({ item }: { item: ContentBlock }) => {
    // Create a virtual 1-block article for fast single-row rendering
    const singleBlockAst = {
      length: 1,
      getBlock: (i: number) => (i === 0 ? item : null),
      toJSON: () => JSON.stringify([item]),
      equals: () => false,
      dispose: () => {},
    } as any;

    return <HtmlRenderer parsedAst={singleBlockAst} />;
  };

  return (
    <FlashList
      data={blocks}
      renderItem={renderItem}
      estimatedItemSize={60}
      keyExtractor={(_, index) => `flash-block-${index}`}
    />
  );
}
```

---

### Recipe 2: Offline Caching with MMKV / SQLite

Cache parsed AST structures across application sessions to eliminate HTML parsing overhead on subsequent launches:

```typescript
import {
  parseHTMLToJSON,
  type ParsedArticleData,
} from 'react-native-fast-html-parser';
import { MMKV } from 'react-native-mmkv';

const storage = new MMKV();

export function fetchAndCacheArticle(
  articleId: string,
  rawHtml: string
): ParsedArticleData {
  const cacheKey = `article_ast_${articleId}`;

  // 1. Check offline cache
  const cachedJson = storage.getString(cacheKey);
  if (cachedJson) {
    return JSON.parse(cachedJson);
  }

  // 2. Parse natively in 1-pass Rust pipeline
  const nativeJsonString = parseHTMLToJSON(rawHtml);

  // 3. Store serialized JSON directly into MMKV
  storage.set(cacheKey, nativeJsonString);

  return JSON.parse(nativeJsonString);
}
```

---

### Recipe 3: Custom Video & Rich Media Player

Intercept `Video` and `Audio` blocks to render native players like `react-native-video`:

```tsx
import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import {
  HtmlRenderer,
  type CustomBlockRenderer,
} from 'react-native-fast-html-parser';

const CustomVideoRenderer: CustomBlockRenderer = ({ block }) => {
  return (
    <View style={styles.videoContainer}>
      <Text style={styles.videoLabel}>Video Stream</Text>
      <Text style={styles.videoUrl}>Source: {block.src}</Text>
      {block.poster ? (
        <Text style={styles.posterText}>Poster: {block.poster}</Text>
      ) : null}
      {block.caption ? (
        <Text style={styles.captionText}>{block.caption}</Text>
      ) : null}
    </View>
  );
};

export function MediaArticle({ html }: { html: string }) {
  return (
    <HtmlRenderer
      html={html}
      renderers={{
        Video: CustomVideoRenderer,
      }}
    />
  );
}

const styles = StyleSheet.create({
  videoContainer: {
    backgroundColor: '#0f172a',
    padding: 16,
    borderRadius: 8,
    marginVertical: 8,
  },
  videoLabel: { color: '#38bdf8', fontWeight: 'bold' },
  videoUrl: { color: '#f8fafc', fontSize: 13, marginTop: 4 },
  posterText: { color: '#94a3b8', fontSize: 12, marginTop: 2 },
  captionText: { color: '#cbd5e1', fontStyle: 'italic', marginTop: 4 },
});
```

---

### Recipe 4: Custom Syntax Highlighting for Code Blocks

```tsx
import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import {
  HtmlRenderer,
  type CustomBlockRenderer,
} from 'react-native-fast-html-parser';

const SyntaxHighlightedCode: CustomBlockRenderer = ({ block }) => {
  return (
    <View style={styles.box}>
      <View style={styles.badge}>
        <Text style={styles.badgeText}>{block.language || 'code'}</Text>
      </View>
      <Text style={styles.codeText}>{block.code}</Text>
    </View>
  );
};

export function CodeArticle({ html }: { html: string }) {
  return (
    <HtmlRenderer
      html={html}
      renderers={{
        CodeBlock: SyntaxHighlightedCode,
      }}
    />
  );
}

const styles = StyleSheet.create({
  box: {
    backgroundColor: '#1e1e1e',
    borderRadius: 8,
    padding: 12,
    marginVertical: 8,
  },
  badge: {
    alignSelf: 'flex-start',
    backgroundColor: '#333333',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
  },
  badgeText: {
    color: '#4ec9b0',
    fontSize: 11,
    fontFamily: 'Courier',
    fontWeight: 'bold',
  },
  codeText: {
    color: '#d4d4d4',
    fontFamily: 'Courier',
    fontSize: 13,
    marginTop: 8,
  },
});
```

---

## 📚 Full TypeScript Type Reference

All data structures, props, and custom renderer types are fully exported:

```typescript
import type {
  // Core HybridObject Interfaces
  InlineNode,
  ListItem,
  TableCell,
  TableRow,
  DefinitionItem,
  ContentBlock,
  ParsedArticle,
  FastHtmlParser,

  // UI Component Props & Types
  HtmlRendererProps,
  VirtualizedHtmlRendererProps,
  CustomBlockRenderer,
  CustomInlineRenderer,

  // Canonical Adapter Types
  CanonicalAdapterConfig,
  BlockTransformer,

  // JSON AST Data Models
  InlineNodeData,
  BaseBlockData,
  HeadingBlockData,
  ParagraphBlockData,
  ListItemData,
  ListBlockData,
  TableCellData,
  TableRowData,
  TableBlockData,
  ImageBlockData,
  CodeBlockData,
  QuoteBlockData,
  CustomBlockData,
  ContentBlockData,
  ParsedArticleData,
} from 'react-native-fast-html-parser';
```

---

## 📋 HTML Compatibility Matrix

For the complete 60+ HTML tag mapping specifications, attributes fidelity table, semantic normalization rules, and test verification suite, see:

👉 **[HTML_COMPATIBILITY_MATRIX.md](./HTML_COMPATIBILITY_MATRIX.md)**

---

## 📄 License

MIT © [abhishekce17](https://github.com/abhishekce17)
