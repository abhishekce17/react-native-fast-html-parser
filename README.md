# react-native-fast-html-parser

A high-performance, zero-copy HTML-to-structured-JSON parser library for React Native. Powered by a lightning-fast Rust core engine and integrated via direct C++ JSI using Margelo Nitro Modules.

It achieves pure zero-copy shared-memory access on-device. This avoids standard string/JSON serialization overhead and Hermes garbage collection pauses during render loops.

---

## Features

- **Blazing Fast**: Directly parses HTML in CPU memory space using Rust's ecosystem.
- **Zero-Copy JSI Access**: Access elements recursively on-demand. Properties are retrieved directly from the native C++ pointer space without copying whole objects.
- **No JSON Intermediates**: Avoids standard CPU intensive parsing overhead of `JSON.parse` or bridge string serialization.
- **Type-safe AST**: Comprehensive typescript interfaces representing paragraph, headings, lists, tables, code blocks, figures, and rich media.

---

## Installation

Run this command inside your React Native application root directory:

```bash
npm install react-native-fast-html-parser
```
or if you use Yarn:
```bash
yarn add react-native-fast-html-parser
```

### iOS Setup
Install pod dependencies:
```bash
cd ios && pod install
```

### Build & Run
Since this package contains native C++ and precompiled Rust libraries, you must rebuild the application:
```bash
npx react-native run-ios
# or
npx react-native run-android
```

---

## Basic Usage

Import the `parseHTML` function to parse raw HTML and safely read properties by checking the `block.type` first:

```typescript
import { parseHTML } from 'react-native-fast-html-parser';

const html = "<h1>Title</h1><p>This is a <b>bold</b> word.</p>";
const article = parseHTML(html);

if (article) {
  console.log("Total Blocks parsed:", article.length); // 2
  
  // Fetch block by index (calls C++ FFI directly)
  const block = article.getBlock(0);
  
  if (block) {
    console.log("Block Type:", block.type); // "Heading"
    
    // 1. Safe Property Querying via Block Type
    if (block.type === 'Heading') {
      console.log("Heading Level:", block.level); // 1 (Safe to read)
    } else if (block.type === 'CodeBlock') {
      console.log("Code language:", block.language); // Safe to read
    }

    // 2. Safe Default Fallbacks
    // If you read a property that does not belong to the block type, 
    // it returns a safe default fallback instead of returning undefined or crashing:
    console.log("Alt Text (non-applicable):", block.alt); // "" (empty string)
    
    // 3. Read inline child nodes
    const inline = block.getChild(0);
    if (inline) {
      console.log("Inline Type:", inline.type); // "Text"
      console.log("Text Content:", inline.text); // "Title"
    }
  }
}
```

---

## Block Type Mapping & Properties

Here is the quick reference of which properties/methods are valid for each `block.type`:

| `block.type` | Applicable Fields & Methods | Description |
| :--- | :--- | :--- |
| **`Paragraph`** | `childCount`, `getChild(index)` | Text block containing inline formatting nodes |
| **`Heading`** | `level` (1-6), `childCount`, `getChild(index)` | Section headers containing formatted inline text |
| **`Image`** / **`Figure`** | `url`, `alt`, `caption` | Images and figure elements with alt text and captions |
| **`CodeBlock`** | `code`, `language` | Syntax-highlighted code blocks |
| **`List`** | `ordered` (boolean), `itemCount`, `getItem(index)` | Ordered (`ol`) or Unordered (`ul`) list elements |
| **`Table`** | `rowCount`, `getRow(index)` | Tables containing rows and cells |
| **`Quote`** | `childCount`, `getQuoteChild(index)` | Blockquotes containing nested block elements |
| **`DefinitionList`** | `itemCount`, `getDefItem(index)` | List of term/definition pairs (`dl`/`dt`/`dd`) |
| **`Video`** | `src`, `poster`, `caption` | Native video components with source, cover, and captions |
| **`Audio`** | `src`, `caption` | Native audio players with captions |
| **`Embed`** | `src`, `title`, `caption` | Generic iframe or external embeds |
| **`Separator`** | *None* | A horizontal rule element (`<hr>`) |

---

## Handling Each Block Type (Code Examples)

Here is how you handle and query properties for each specific block type returned by `article.getBlock(index)`:

### 1. Paragraph & Heading
Paragraphs and Headings contain child inline formatting nodes:
```typescript
if (block.type === 'Paragraph') {
  for (let i = 0; i < block.childCount; i++) {
    const inline = block.getChild(i);
    if (inline) {
      console.log(`Text: ${inline.text}, Formatting: ${inline.type}`);
    }
  }
}

if (block.type === 'Heading') {
  console.log("Heading Level:", block.level); // 1-6
  const firstInlineChild = block.getChild(0);
}
```

### 2. Image, Figure & Media
```typescript
if (block.type === 'Image' || block.type === 'Figure') {
  console.log("URL:", block.url);
  console.log("Alt Text:", block.alt);
  console.log("Caption:", block.caption);
}

if (block.type === 'Video') {
  console.log("Video source:", block.src);
  console.log("Video cover poster:", block.poster);
  console.log("Caption:", block.caption);
}

if (block.type === 'Audio') {
  console.log("Audio source:", block.src);
}

if (block.type === 'Embed') {
  console.log("Iframe source url:", block.src);
  console.log("Iframe title:", block.title);
}
```

### 3. CodeBlock
```typescript
if (block.type === 'CodeBlock') {
  console.log("Raw Code:", block.code);
  console.log("Language:", block.language); // e.g. "typescript", "rust"
}
```

### 4. List (Ordered / Unordered)
```typescript
if (block.type === 'List') {
  console.log("Is ordered list?", block.ordered);
  
  for (let i = 0; i < block.itemCount; i++) {
    const item = block.getItem(i);
    if (item) {
      // 1. Read inline children of list item
      for (let j = 0; j < item.childCount; j++) {
        const inline = item.getChild(j);
        console.log(inline?.text);
      }
      
      // 2. Read nested lists (for multi-level nested lists)
      for (let k = 0; k < item.nestedCount; k++) {
        const nestedBlock = item.getNested(k);
        console.log("Nested block type:", nestedBlock?.type); // "List"
      }
    }
  }
}
```

### 5. Table
```typescript
if (block.type === 'Table') {
  for (let r = 0; r < block.rowCount; r++) {
    const row = block.getRow(r);
    if (row) {
      for (let c = 0; c < row.cellCount; c++) {
        const cell = row.getCell(c);
        if (cell) {
          // Read cell contents
          const inlineChild = cell.getChild(0);
          console.log(`Cell [${r}, ${c}]:`, inlineChild?.text);
        }
      }
    }
  }
}
```

### 6. Quote (Blockquote)
```typescript
if (block.type === 'Quote') {
  // Quote blocks contain sub-blocks (like nested Paragraphs)
  for (let i = 0; i < block.childCount; i++) {
    const quoteChildBlock = block.getQuoteChild(i);
    if (quoteChildBlock) {
      console.log("Nested quote block type:", quoteChildBlock.type); // e.g., "Paragraph"
    }
  }
}
```

### 7. DefinitionList
```typescript
if (block.type === 'DefinitionList') {
  for (let i = 0; i < block.itemCount; i++) {
    const defItem = block.getDefItem(i);
    if (defItem) {
      // Read term inlines (<dt>)
      const termInline = defItem.getTerm(0);
      
      // Read definition inlines (<dd>)
      const defInline = defItem.getDef(0);
    }
  }
}
```

### 8. Separator
```typescript
if (block.type === 'Separator') {
  console.log("Renders horizontal rule line (<hr>)");
}
```

---

## Rendering Components (Traversing the AST)

Here is a complete, production-grade React Native component showing how to traverse the AST recursively to map HTML content to native layout blocks:

```tsx
import React from 'react';
import { View, Text, StyleSheet, ScrollView } from 'react-native';
import { parseHTML, type ContentBlock, type InlineNode } from 'react-native-fast-html-parser';

// Helper to render formatting inlines (bold, italic, links)
function RenderInline({ node }: { node: InlineNode }): React.ReactElement {
  if (node.type === 'Text') {
    return <Text>{node.text}</Text>;
  }
  if (node.type === 'Bold') {
    return <Text style={styles.bold}>{node.text}</Text>;
  }
  if (node.type === 'Italic') {
    return <Text style={styles.italic}>{node.text}</Text>;
  }
  if (node.type === 'Link') {
    return <Text style={styles.link} onPress={() => console.log('Open URL:', node.url)}>{node.text}</Text>;
  }
  return <Text>{node.text}</Text>;
}

// Helper to render block-level structures
function RenderBlock({ block }: { block: ContentBlock }): React.ReactElement | null {
  switch (block.type) {
    case 'Heading': {
      const children = [];
      for (let i = 0; i < block.childCount; i++) {
        const child = block.getChild(i);
        if (child) children.push(<RenderInline key={i} node={child} />);
      }
      const isH1 = block.level === 1;
      return <Text style={isH1 ? styles.h1 : styles.h2}>{children}</Text>;
    }

    case 'Paragraph': {
      const children = [];
      for (let i = 0; i < block.childCount; i++) {
        const child = block.getChild(i);
        if (child) children.push(<RenderInline key={i} node={child} />);
      }
      return <Text style={styles.paragraph}>{children}</Text>;
    }

    case 'CodeBlock': {
      return (
        <View style={styles.codeContainer}>
          <Text style={styles.codeLanguage}>{block.language || 'code'}</Text>
          <Text style={styles.codeText}>{block.code}</Text>
        </View>
      );
    }

    case 'Quote': {
      const quoteChildren = [];
      for (let i = 0; i < block.childCount; i++) {
        const quoteBlock = block.getQuoteChild(i);
        if (quoteBlock) {
          quoteChildren.push(<RenderBlock key={i} block={quoteBlock} />);
        }
      }
      return <View style={styles.quoteBorder}>{quoteChildren}</View>;
    }

    default:
      return null;
  }
}

// Main Renderer Component
export default function HtmlContentRenderer({ html }: { html: string }) {
  const article = React.useMemo(() => parseHTML(html), [html]);
  
  if (!article) {
    return <Text>Error parsing content</Text>;
  }

  const blocks = [];
  for (let i = 0; i < article.length; i++) {
    const block = article.getBlock(i);
    if (block) {
      blocks.push(<RenderBlock key={i} block={block} />);
    }
  }

  return <ScrollView style={styles.container}>{blocks}</ScrollView>;
}

const styles = StyleSheet.create({
  container: { flex: 1, padding: 16, backgroundColor: '#ffffff' },
  paragraph: { fontSize: 16, lineHeight: 24, color: '#333333', marginVertical: 8 },
  h1: { fontSize: 24, fontWeight: 'bold', marginVertical: 12, color: '#111111' },
  h2: { fontSize: 20, fontWeight: 'bold', marginVertical: 10, color: '#222222' },
  bold: { fontWeight: 'bold' },
  italic: { fontStyle: 'italic' },
  link: { color: '#0066cc', textDecorationLine: 'underline' },
  codeContainer: { backgroundColor: '#f5f5f5', padding: 12, borderRadius: 8, marginVertical: 8 },
  codeLanguage: { fontSize: 11, color: '#888888', textTransform: 'uppercase', marginBottom: 4 },
  codeText: { fontFamily: 'Courier', fontSize: 14, color: '#333333' },
  quoteBorder: { borderLeftWidth: 4, borderLeftColor: '#cccccc', paddingLeft: 12, marginVertical: 8, fontStyle: 'italic' }
});
```

---

## AST Type Reference

All exposed interfaces are fully registered with JSI and typechecked:

```typescript
export interface InlineNode {
  readonly type: 'Text' | 'Bold' | 'Italic' | 'Link' | 'Strikethrough' | 'Underline' | 'Code';
  readonly text: string;
  readonly url: string;
  readonly childCount: number;
  getChild(index: number): InlineNode | null;
}

export interface ListItem {
  readonly childCount: number;
  getChild(index: number): InlineNode | null;
  readonly nestedCount: number;
  getNested(index: number): ContentBlock | null;
}

export interface TableCell {
  readonly childCount: number;
  getChild(index: number): InlineNode | null;
}

export interface TableRow {
  readonly cellCount: number;
  getCell(index: number): TableCell | null;
}

export interface DefinitionItem {
  readonly termCount: number;
  getTerm(index: number): InlineNode | null;
  readonly defCount: number;
  getDef(index: number): InlineNode | null;
}

export interface ContentBlock {
  readonly type: 'Paragraph' | 'Heading' | 'List' | 'Table' | 'Quote' | 'CodeBlock' | 'Media' | 'DefinitionList' | 'Embed';
  readonly level: number;       // For Heading elements (1-6)
  readonly url: string;         // For Image/Media nodes
  readonly alt: string;         // For Image/Media nodes
  readonly caption: string;     // For Figures
  readonly code: string;        // For Code blocks
  readonly language: string;    // For Code blocks
  readonly src: string;         // For Video/Audio/Embed source url
  readonly poster: string;      // For Video poster url
  readonly title: string;       // For Embed elements
  
  // Children elements
  readonly childCount: number;
  getChild(index: number): InlineNode | null;
  getQuoteChild(index: number): ContentBlock | null;

  // List properties
  readonly ordered: boolean;
  readonly itemCount: number;
  getItem(index: number): ListItem | null;

  // Table properties
  readonly rowCount: number;
  getRow(index: number): TableRow | null;

  // DefinitionList properties
  getDefItem(index: number): DefinitionItem | null;
}

export interface ParsedArticle {
  readonly length: number;
  getBlock(index: number): ContentBlock | null;
}

export interface FastHtmlParser {
  parse(html: string): ParsedArticle | null;
}
```

---

## License

MIT © [abhishekce17](https://github.com/abhishekce17)
