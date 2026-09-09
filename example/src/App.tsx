/**
 * react-native-fast-html-parser — Comprehensive Example App
 *
 * Tabs:
 *  1. Rendered HTML      — HtmlRenderer with tagsStyles + custom block/inline renderers
 *  2. Virtualized        — VirtualizedHtmlRenderer for long documents
 *  3. Wrappers           — getBlocks / getChildren / getItems / getRows / getCells /
 *                          getQuoteChildren / getDefItems used directly
 *  4. JSON Pipeline      — parseHTMLToJSON / article.toJSON() / createCanonicalAdapter
 */

import { useMemo, useState } from 'react';
import {
  Alert,
  SafeAreaView,
  ScrollView,
  StyleSheet,
  Text,
  TouchableOpacity,
  View,
  type TextStyle,
} from 'react-native';
import {
  createCanonicalAdapter,
  getBlocks,
  getCells,
  getChildren,
  getDefItems,
  getItems,
  getNestedBlocks,
  getQuoteChildren,
  getRows,
  HtmlRenderer,
  parseHTML,
  parseHTMLToJSON,
  VirtualizedHtmlRenderer,
  type ContentBlock,
  type CustomBlockRenderer,
  type CustomInlineRenderer,
  type ParsedArticle,
  type ParsedArticleData,
} from 'react-native-fast-html-parser';

// ─── Shared HTML samples ─────────────────────────────────────────────────────

const RICH_HTML = `
<h1>⚡ react-native-fast-html-parser</h1>
<p>A high-performance HTML pipeline powered by a compiled <b>Rust</b> core
and direct <i>C++ JSI</i> via <a href="https://nitro.margelo.com">Nitro Modules</a>.</p>

<blockquote>
  <p>"Zero-copy JSI access bypasses bridge serialization entirely."</p>
  <p>— Architecture Design Doc</p>
</blockquote>

<h2>Features</h2>
<ul>
  <li>Sub-millisecond native parsing</li>
  <li>Lazy JSI HybridObject tree — no full AST in JS heap</li>
  <li>1-pass <code>parseHTMLToJSON()</code> for caching
    <ol>
      <li>SQLite / MMKV storage</li>
      <li>Redux / Zustand state slices</li>
    </ol>
  </li>
  <li>Drop-in <code>&lt;HtmlRenderer /&gt;</code> component</li>
</ul>

<h2>Benchmark (100 KB payload)</h2>
<table>
  <tr><th>Metric</th><th>Value</th></tr>
  <tr><td>Parse time</td><td>1.794 ms</td></tr>
  <tr><td>JSON serialization</td><td>0.137 ms</td></tr>
  <tr><td>Throughput</td><td>50.63 MB/s</td></tr>
  <tr><td>Blocks extracted</td><td>674 blocks</td></tr>
</table>

<h2>Quick Start</h2>
<pre><code class="typescript">import { HtmlRenderer } from 'react-native-fast-html-parser';

export function ArticleScreen({ html }: { html: string }) {
  return &lt;HtmlRenderer html={html} /&gt;;
}</code></pre>

<h2>Definition List</h2>
<dl>
  <dt>JSI</dt><dd>JavaScript Interface — direct C++ ↔ JS bridge, no serialization.</dd>
  <dt>Nitro Modules</dt><dd>Next-gen RN native module framework built on JSI.</dd>
  <dt>HybridObject</dt><dd>C++ object with a JS-facing facade, zero memory copy.</dd>
</dl>

<figure>
  <img src="https://picsum.photos/seed/rn-parser/800/300" alt="Architecture diagram" />
  <figcaption>Rust parser → C++ JSI bridge → React Native UI</figcaption>
</figure>

<hr />

<p>Built with ❤️ for the React Native community.</p>
`;

// Long document for Virtualized tab — 40+ paragraphs
const LONG_HTML = Array.from(
  { length: 40 },
  (_, i) => `
<h${(i % 3) + 2}>Section ${i + 1}: Native Performance</h${(i % 3) + 2}>
<p>This is paragraph ${i + 1}. The <b>VirtualizedHtmlRenderer</b> uses
<i>FlatList</i> row recycling, so only the visible blocks are mounted in React.
Long articles of any length stay at <code>120 FPS</code> scroll.</p>
${i % 5 === 0 ? `<blockquote><p>Virtualization milestone at block ${i + 1}.</p></blockquote>` : ''}
${i % 7 === 0 ? `<ul><li>Item A in section ${i + 1}</li><li>Item B</li></ul>` : ''}
`
).join('');

// ─── Tab navigation ──────────────────────────────────────────────────────────

const TABS = ['Rendered', 'Virtualized', 'Wrappers', 'JSON'] as const;
type Tab = (typeof TABS)[number];

// ─── Custom block renderers ───────────────────────────────────────────────────

const CustomCodeBlock: CustomBlockRenderer = ({ block }) => (
  <View style={styles.customCode}>
    {block.language ? (
      <View style={styles.codeBadge}>
        <Text style={styles.codeBadgeText}>{block.language.toUpperCase()}</Text>
      </View>
    ) : null}
    <ScrollView horizontal showsHorizontalScrollIndicator={false}>
      <Text style={styles.codeText}>{block.code}</Text>
    </ScrollView>
  </View>
);

// ─── Custom inline renderers ──────────────────────────────────────────────────

const CustomLink: CustomInlineRenderer = ({ node }) => (
  <Text
    style={styles.customLink}
    onPress={() => Alert.alert('Link pressed', node.url)}
    accessibilityRole="link"
  >
    {node.text ||
      getChildren(null as any)
        .map((c) => c.text)
        .join('')}
  </Text>
);

const CustomBold: CustomInlineRenderer = ({ node }) => (
  <Text style={styles.customBold}>{node.text}</Text>
);

// ─── Tab 1: HtmlRenderer ─────────────────────────────────────────────────────

function RenderedTab({ parsedAst }: { parsedAst: ParsedArticle | null }) {
  return (
    <ScrollView contentContainerStyle={styles.tabContent}>
      <Text style={styles.sectionLabel}>
        Using &lt;HtmlRenderer parsedAst=&#123;…&#125; /&gt;
      </Text>
      <Text style={styles.sectionHint}>
        Custom CodeBlock renderer, custom Bold/Link inline renderers, tagsStyles
        overrides.
      </Text>
      <View style={styles.card}>
        <HtmlRenderer
          parsedAst={parsedAst}
          tagsStyles={{
            h1: {
              fontSize: 24,
              color: '#7c3aed',
              fontWeight: '800',
            } as TextStyle,
            h2: {
              fontSize: 18,
              color: '#1e40af',
              fontWeight: '700',
            } as TextStyle,
            a: { color: '#2563eb' } as TextStyle,
            blockquote: { borderLeftColor: '#7c3aed' },
          }}
          renderers={{ CodeBlock: CustomCodeBlock }}
          inlineRenderers={{ Bold: CustomBold, Link: CustomLink }}
          onLinkPress={(url) => Alert.alert('onLinkPress', url)}
        />
      </View>
    </ScrollView>
  );
}

// ─── Tab 2: VirtualizedHtmlRenderer ──────────────────────────────────────────

function VirtualizedTab() {
  const longAst = useMemo(() => parseHTML(LONG_HTML), []);

  return (
    <VirtualizedHtmlRenderer
      parsedAst={longAst}
      baseStyle={{ fontSize: 15, color: '#1e293b', lineHeight: 24 }}
      tagsStyles={{
        h2: { color: '#0369a1', fontWeight: '700' } as TextStyle,
        h3: { color: '#0891b2', fontWeight: '600' } as TextStyle,
        h4: { color: '#0e7490' } as TextStyle,
        code: { backgroundColor: '#f0fdf4', color: '#166534' } as TextStyle,
      }}
      contentContainerStyle={styles.virtualContent}
      ListHeaderComponent={
        <View style={styles.virtualHeader}>
          <Text style={styles.virtualHeaderTitle}>VirtualizedHtmlRenderer</Text>
          <Text style={styles.virtualHeaderSub}>
            {getBlocks(parseHTML(LONG_HTML)).length} blocks · FlatList recycling
            · 120 FPS
          </Text>
        </View>
      }
      ListFooterComponent={
        <View style={styles.virtualFooter}>
          <Text style={styles.virtualFooterText}>
            ✅ End of document — all {getBlocks(parseHTML(LONG_HTML)).length}{' '}
            blocks rendered.
          </Text>
        </View>
      }
    />
  );
}

// ─── Tab 3: Wrappers ─────────────────────────────────────────────────────────

interface WrapperRow {
  label: string;
  value: string;
}

function WrappersTab({ parsedAst }: { parsedAst: ParsedArticle | null }) {
  const rows = useMemo<WrapperRow[]>(() => {
    if (!parsedAst) return [];

    // getBlocks
    const blocks = getBlocks(parsedAst);
    const result: WrapperRow[] = [
      { label: 'getBlocks(article).length', value: String(blocks.length) },
    ];

    // getChildren — from first Paragraph
    const para = blocks.find((b: ContentBlock) => b.type === 'Paragraph');
    if (para) {
      const inlines = getChildren(para);
      result.push({
        label: 'getChildren(paragraph).length',
        value: String(inlines.length),
      });
      result.push({
        label: 'getChildren types',
        value: inlines.map((n) => n.type).join(', '),
      });
    }

    // getItems — from first List
    const list = blocks.find((b: ContentBlock) => b.type === 'List');
    if (list) {
      const items = getItems(list);
      result.push({
        label: 'getItems(list).length',
        value: String(items.length),
      });

      // getNestedBlocks — from items
      items.forEach((item, i) => {
        const nested = getNestedBlocks(item);
        if (nested.length > 0) {
          result.push({
            label: `getNestedBlocks(item[${i}]).length`,
            value: String(nested.length),
          });
        }
      });
    }

    // getRows + getCells — from Table
    const table = blocks.find((b: ContentBlock) => b.type === 'Table');
    if (table) {
      const rows2 = getRows(table);
      result.push({
        label: 'getRows(table).length',
        value: String(rows2.length),
      });
      if (rows2[0]) {
        const cells = getCells(rows2[0]);
        result.push({
          label: 'getCells(row[0]).length',
          value: String(cells.length),
        });
        result.push({
          label: 'Cell[0] text',
          value: getChildren(cells[0] ?? null)
            .map((c) => c.text)
            .join(''),
        });
      }
    }

    // getQuoteChildren — from Quote
    const quote = blocks.find((b: ContentBlock) => b.type === 'Quote');
    if (quote) {
      const qc = getQuoteChildren(quote);
      result.push({
        label: 'getQuoteChildren(quote).length',
        value: String(qc.length),
      });
    }

    // getDefItems — from DefinitionList
    const dl = blocks.find((b: ContentBlock) => b.type === 'DefinitionList');
    if (dl) {
      const defs = getDefItems(dl);
      result.push({
        label: 'getDefItems(dl).length',
        value: String(defs.length),
      });
      if (defs[0]) {
        result.push({
          label: 'defItems[0] term',
          value: defs[0].getTerm(0)?.text ?? '—',
        });
        result.push({
          label: 'defItems[0] definition',
          value: defs[0].getDef(0)?.text
            ? defs[0].getDef(0)!.text.slice(0, 40) + '…'
            : '—',
        });
      }
    }

    return result;
  }, [parsedAst]);

  return (
    <ScrollView contentContainerStyle={styles.tabContent}>
      <Text style={styles.sectionLabel}>AST Traversal Wrappers</Text>
      <Text style={styles.sectionHint}>
        getBlocks · getChildren · getItems · getNestedBlocks · getRows ·
        getCells · getQuoteChildren · getDefItems — all called on the same
        ParsedArticle.
      </Text>
      {rows.map((row, i) => (
        <View key={i} style={styles.wrapperRow}>
          <Text style={styles.wrapperLabel}>{row.label}</Text>
          <Text style={styles.wrapperValue}>{row.value}</Text>
        </View>
      ))}
    </ScrollView>
  );
}

// ─── Tab 4: JSON Pipeline ─────────────────────────────────────────────────────

function JsonTab({ parsedAst }: { parsedAst: ParsedArticle | null }) {
  const [mode, setMode] = useState<'parseHTMLToJSON' | 'toJSON' | 'adapter'>(
    'parseHTMLToJSON'
  );

  // parseHTMLToJSON — 1-pass Rust pipeline
  const directJson = useMemo(() => parseHTMLToJSON(RICH_HTML), []);

  // article.toJSON() — serialize existing HybridObject
  const articleJson = useMemo(() => {
    if (!parsedAst) return '';
    return parsedAst.toJSON();
  }, [parsedAst]);

  // createCanonicalAdapter — domain schema mapping
  const adapter = useMemo(
    () =>
      createCanonicalAdapter<
        { totalBlocks: number; headings: string[]; sections: any[] },
        { kind: string; summary: string }
      >({
        transformers: {
          heading: (block, _i) => ({
            kind: 'heading',
            summary: `H${(block as any).level}: ${(block as any).children?.map((c: any) => c.text).join('') ?? ''}`,
          }),
          paragraph: (block, _i) => ({
            kind: 'paragraph',
            summary: `${(block as any).children
              ?.map((c: any) => c.text)
              .join('')
              .slice(0, 60)}…`,
          }),
        },
        transformBlock: (block) => ({ kind: block.type, summary: block.type }),
        transformArticle: (_article, domainBlocks) => ({
          totalBlocks: domainBlocks.length,
          headings: domainBlocks
            .filter((b) => b.kind === 'heading')
            .map((b) => b.summary),
          sections: domainBlocks,
        }),
      }),
    []
  );

  const adapterOutput = useMemo(() => {
    const parsed: ParsedArticleData = JSON.parse(directJson);
    return JSON.stringify(adapter.adapt(parsed), null, 2);
  }, [directJson, adapter]);

  const displayJson =
    mode === 'parseHTMLToJSON'
      ? directJson
      : mode === 'toJSON'
        ? articleJson
        : adapterOutput;

  return (
    <View style={styles.jsonTab}>
      {/* Mode switcher */}
      <View style={styles.jsonModeRow}>
        {(
          [
            ['parseHTMLToJSON', '1-Pass JSON'],
            ['toJSON', 'article.toJSON()'],
            ['adapter', 'Adapter'],
          ] as const
        ).map(([key, label]) => (
          <TouchableOpacity
            key={key}
            style={[styles.modeBtn, mode === key && styles.activeModeBtn]}
            onPress={() => setMode(key)}
          >
            <Text
              style={[
                styles.modeBtnText,
                mode === key && styles.activeModeBtnText,
              ]}
            >
              {label}
            </Text>
          </TouchableOpacity>
        ))}
      </View>

      {/* Description */}
      <Text style={styles.jsonDesc}>
        {mode === 'parseHTMLToJSON'
          ? 'parseHTMLToJSON(html) — 1-pass Rust serialization. Native memory freed instantly. Best for MMKV/SQLite caching.'
          : mode === 'toJSON'
            ? 'article.toJSON() — serializes an existing ParsedArticle HybridObject. Use when you rendered first and then want to cache.'
            : 'createCanonicalAdapter() — maps parser blocks to your domain schema (headings extracted, paragraph summaries, etc).'}
      </Text>

      {/* JSON output */}
      <ScrollView
        style={styles.jsonScroll}
        contentContainerStyle={styles.jsonScrollContent}
      >
        <Text style={styles.jsonText}>{displayJson}</Text>
      </ScrollView>
    </View>
  );
}

// ─── Root App ─────────────────────────────────────────────────────────────────

export default function App() {
  const [activeTab, setActiveTab] = useState<Tab>('Rendered');

  // Parse once — shared across Rendered, Wrappers, and JSON tabs
  const parsedAst = useMemo(() => parseHTML(RICH_HTML), []);
  const blockCount = useMemo(() => getBlocks(parsedAst).length, [parsedAst]);

  return (
    <SafeAreaView style={styles.root}>
      {/* Header */}
      <View style={styles.header}>
        <Text style={styles.headerTitle}>react-native-fast-html-parser</Text>
        <Text style={styles.headerSub}>
          {blockCount} blocks · Rust core · JSI
        </Text>
      </View>

      {/* Tab bar */}
      <View style={styles.tabBar}>
        {TABS.map((tab) => (
          <TouchableOpacity
            key={tab}
            style={[styles.tabBtn, activeTab === tab && styles.activeTabBtn]}
            onPress={() => setActiveTab(tab)}
          >
            <Text
              style={[
                styles.tabBtnText,
                activeTab === tab && styles.activeTabBtnText,
              ]}
            >
              {tab}
            </Text>
          </TouchableOpacity>
        ))}
      </View>

      {/* Active tab */}
      <View style={styles.tabBody}>
        {activeTab === 'Rendered' && <RenderedTab parsedAst={parsedAst} />}
        {activeTab === 'Virtualized' && <VirtualizedTab />}
        {activeTab === 'Wrappers' && <WrappersTab parsedAst={parsedAst} />}
        {activeTab === 'JSON' && <JsonTab parsedAst={parsedAst} />}
      </View>
    </SafeAreaView>
  );
}

// ─── Styles ───────────────────────────────────────────────────────────────────

const styles = StyleSheet.create({
  root: {
    flex: 1,
    backgroundColor: '#f8fafc',
  },

  // Header
  header: {
    backgroundColor: '#7c3aed',
    paddingVertical: 14,
    paddingHorizontal: 18,
  },
  headerTitle: {
    fontSize: 16,
    fontWeight: '800',
    color: '#ffffff',
    letterSpacing: 0.3,
  },
  headerSub: {
    fontSize: 12,
    color: '#ddd6fe',
    marginTop: 2,
  },

  // Tab bar
  tabBar: {
    flexDirection: 'row',
    backgroundColor: '#ffffff',
    borderBottomWidth: 1,
    borderBottomColor: '#e2e8f0',
  },
  tabBtn: {
    flex: 1,
    paddingVertical: 11,
    alignItems: 'center',
  },
  activeTabBtn: {
    borderBottomWidth: 2,
    borderBottomColor: '#7c3aed',
  },
  tabBtnText: {
    fontSize: 13,
    fontWeight: '600',
    color: '#94a3b8',
  },
  activeTabBtnText: {
    color: '#7c3aed',
  },
  tabBody: {
    flex: 1,
  },

  // Tab content shared
  tabContent: {
    padding: 16,
    paddingBottom: 40,
  },
  sectionLabel: {
    fontSize: 13,
    fontWeight: '700',
    color: '#7c3aed',
    marginBottom: 4,
    textTransform: 'uppercase',
    letterSpacing: 0.5,
  },
  sectionHint: {
    fontSize: 12,
    color: '#64748b',
    marginBottom: 12,
    lineHeight: 18,
  },
  card: {
    backgroundColor: '#ffffff',
    borderRadius: 12,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 6,
    elevation: 2,
  },

  // Custom CodeBlock renderer
  customCode: {
    backgroundColor: '#0f172a',
    borderRadius: 8,
    padding: 12,
    marginVertical: 8,
  },
  codeBadge: {
    alignSelf: 'flex-start',
    backgroundColor: '#1e293b',
    borderRadius: 4,
    paddingHorizontal: 6,
    paddingVertical: 2,
    marginBottom: 8,
  },
  codeBadgeText: {
    color: '#38bdf8',
    fontSize: 10,
    fontWeight: '700',
  },
  codeText: {
    fontFamily: 'Courier',
    color: '#f8fafc',
    fontSize: 12,
    lineHeight: 18,
  },

  // Custom inline renderers
  customLink: {
    color: '#7c3aed',
    fontWeight: '700',
    textDecorationLine: 'underline',
  },
  customBold: {
    fontWeight: '900',
    color: '#0f172a',
  },

  // Virtualized tab
  virtualContent: {
    paddingHorizontal: 16,
    paddingBottom: 40,
  },
  virtualHeader: {
    backgroundColor: '#7c3aed',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  virtualHeaderTitle: {
    fontSize: 18,
    fontWeight: '800',
    color: '#ffffff',
  },
  virtualHeaderSub: {
    fontSize: 13,
    color: '#ddd6fe',
    marginTop: 4,
  },
  virtualFooter: {
    paddingVertical: 24,
    alignItems: 'center',
  },
  virtualFooterText: {
    fontSize: 13,
    color: '#64748b',
  },

  // Wrappers tab
  wrapperRow: {
    backgroundColor: '#ffffff',
    borderRadius: 8,
    paddingHorizontal: 14,
    paddingVertical: 10,
    marginBottom: 8,
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.04,
    shadowRadius: 3,
    elevation: 1,
  },
  wrapperLabel: {
    fontSize: 12,
    fontFamily: 'Courier',
    color: '#475569',
    flex: 1,
    flexWrap: 'wrap',
  },
  wrapperValue: {
    fontSize: 13,
    fontWeight: '700',
    color: '#7c3aed',
    marginLeft: 10,
    flexShrink: 1,
    textAlign: 'right',
  },

  // JSON tab
  jsonTab: {
    flex: 1,
    backgroundColor: '#0f172a',
  },
  jsonModeRow: {
    flexDirection: 'row',
    backgroundColor: '#1e293b',
    padding: 6,
    gap: 6,
  },
  modeBtn: {
    flex: 1,
    paddingVertical: 8,
    borderRadius: 6,
    alignItems: 'center',
  },
  activeModeBtn: {
    backgroundColor: '#7c3aed',
  },
  modeBtnText: {
    fontSize: 11,
    fontWeight: '600',
    color: '#94a3b8',
  },
  activeModeBtnText: {
    color: '#ffffff',
  },
  jsonDesc: {
    fontSize: 11,
    color: '#64748b',
    paddingHorizontal: 14,
    paddingVertical: 8,
    lineHeight: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#1e293b',
  },
  jsonScroll: {
    flex: 1,
  },
  jsonScrollContent: {
    padding: 14,
  },
  jsonText: {
    fontFamily: 'Courier',
    fontSize: 11,
    color: '#94a3b8',
    lineHeight: 18,
  },
});
