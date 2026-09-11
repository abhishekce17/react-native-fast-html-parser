/**
 * react-native-fast-html-parser — Comprehensive Example App
 *
 * Tabs:
 *  1. FastHtmlView       — Native Fabric RichText Engine with tagsStyles + Custom Renderer Injection
 *  2. Infinite Scale     — Massive 40+ paragraph HTML doc rendered 100% natively at 120 FPS
 *  3. Wrappers           — getBlocks / getChildren / getItems / getRows / getCells /
 *                          getQuoteChildren / getDefItems used directly
 *  4. JSON Pipeline      — parseHTMLToJSON / article.toJSON() / createCanonicalAdapter
 */

import { useMemo, useState, useCallback } from 'react';
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
  FastHtmlView,
  parseHTML,
  parseHTMLAsync,
  parseHTMLToJSON,
  type ContentBlock,
  type CustomBlockRenderer,
  type ParsedArticle,
  type ParsedArticleData,
} from 'react-native-fast-html-parser';

// ─── Shared HTML samples ─────────────────────────────────────────────────────

const RICH_HTML = `
<h1>⚡ react-native-fast-html-parser</h1>
<p>A high-performance HTML pipeline powered by a compiled <b>C++ (Lexbor)</b> core
and direct <i>C++ JSI</i> via <a href="https://nitro.margelo.com">Nitro Modules</a>.</p>

<blockquote>
  <p>"Zero-copy JSI access bypasses bridge serialization entirely."</p>
  <p>— Architecture Design Doc</p>
</blockquote>

<h2>Features</h2>
<ul>
  <li>Sub-millisecond native parsing</li>
  <li>100% Native Fabric RichText View (TextKit 2 & Android Spannables)</li>
  <li>Continuous text selection across paragraphs, headings & lists</li>
  <li>1-pass <code>parseHTMLToJSON()</code> for caching
    <ol>
      <li>SQLite / MMKV storage</li>
      <li>Redux / Zustand state slices</li>
    </ol>
  </li>
  <li>Drop-in <code>&lt;FastHtmlView /&gt;</code> component</li>
</ul>

<h2>Benchmark (100 KB payload)</h2>
<table>
  <tr><th>Metric</th><th>Value</th></tr>
  <tr><td>Parse time</td><td>0.353 ms</td></tr>
  <tr><td>JSON serialization</td><td>0.128 ms</td></tr>
  <tr><td>Throughput</td><td>204.48 MB/s</td></tr>
  <tr><td>Blocks extracted</td><td>488 blocks</td></tr>
</table>

<h2>Quick Start</h2>
<pre><code class="typescript">import { FastHtmlView } from 'react-native-fast-html-parser';

export function ArticleScreen({ html }: { html: string }) {
  return &lt;FastHtmlView html={html} /&gt;;
}</code></pre>

<h2>Definition List</h2>
<dl>
  <dt>JSI</dt><dd>JavaScript Interface — direct C++ ↔ JS bridge, no serialization.</dd>
  <dt>Nitro Modules</dt><dd>Next-gen RN native module framework built on JSI.</dd>
  <dt>HybridObject</dt><dd>C++ object with a JS-facing facade, zero memory copy.</dd>
</dl>

<figure>
  <img src="https://picsum.photos/seed/rn-parser/800/300" alt="Architecture diagram" />
  <figcaption>Lexbor C++ parser → Nitro JSI bridge → React Native UI</figcaption>
</figure>

<hr />

<p>Built with ❤️ for the React Native community.</p>
`;

const NEW_FEATURES_HTML = `
<style>
  .highlight { color: #8b5cf6; font-weight: bold; }
  .box { background-color: #f1f5f9; padding: 8px; border-left: 4px solid #8b5cf6; }
</style>

<h2>🔥 All 11 Native Optimizations Live</h2>

<div class="box">
  <p class="highlight">✨ Embedded CSS &lt;style&gt; Sheet Engine in C++ (Lexbor)</p>
  <p>Class selectors, compound rules, and element cascades are resolved in C++ at parse-time with 0 JS overhead.</p>
</div>

<h3>1. HTML5 Named & Numeric Entities</h3>
<p>Entity decoding in C++ ($O(1)$ static table): &ldquo;Double Quotes&rdquo;, &mdash; (em-dash), &hellip; (ellipsis), &euro;100 (Euro), &infin; (Infinity), &copy; 2026, &Delta; (Delta), &#9733; (Star), &hearts; (Hearts).</p>

<h3>2. OpenType Numeric & Tabular Figures</h3>
<p>Invoice #98214: Total = $1,429.50 | 1/2 + 3/4 = 5/4 (Fractions & Tabular Numbers)</p>

<h3>3. Wide Data Table (Horizontal Scroll)</h3>
<table>
  <tr><th>ID</th><th>Service</th><th>Latency</th><th>Throughput</th><th>Status</th><th>Region</th><th>Uptime</th><th>Score</th></tr>
  <tr><td>001</td><td>Lexbor C++</td><td>0.08ms</td><td>150 MB/s</td><td>Active</td><td>us-east</td><td>99.99%</td><td>100</td></tr>
  <tr><td>002</td><td>Nitro JSI</td><td>0.01ms</td><td>950 MB/s</td><td>Active</td><td>eu-central</td><td>99.999%</td><td>100</td></tr>
  <tr><td>003</td><td>TextKit 2</td><td>0.45ms</td><td>60 FPS</td><td>Active</td><td>ap-south</td><td>99.95%</td><td>98</td></tr>
</table>
`;

// Long document for Infinite Scale tab — 40+ paragraphs
const LONG_HTML = Array.from(
  { length: 40 },
  (_, i) => `
<h${(i % 3) + 2}>Section ${i + 1}: Native Performance</h${(i % 3) + 2}>
<p>This is paragraph ${i + 1}. The <b>FastHtmlView</b> uses
native text fragment rendering with <b>0 React Virtual DOM nodes</b>.
Long articles of any length stay at <code>120 FPS</code> smooth scrolling with continuous text selection.</p>
${i % 5 === 0 ? `<blockquote><p>Native milestone at block ${i + 1}.</p></blockquote>` : ''}
${i % 7 === 0 ? `<ul><li>Item A in section ${i + 1}</li><li>Item B</li></ul>` : ''}
`
).join('');

// ─── Tab navigation ──────────────────────────────────────────────────────────

const TABS = [
  'FastHtmlView',
  'New Features',
  'Infinite Scale',
  'Wrappers',
  'JSON',
] as const;
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

// ─── Tab 1: FastHtmlView ─────────────────────────────────────────────────────

function RenderedTab({ parsedAst }: { parsedAst: ParsedArticle | null }) {
  return (
    <ScrollView contentContainerStyle={styles.tabContent}>
      <Text style={styles.sectionLabel}>
        Using &lt;FastHtmlView parsedAst=&#123;…&#125; /&gt;
      </Text>
      <Text style={styles.sectionHint}>
        100% Native Fabric Text Rendering, continuous selection, custom
        CodeBlock injector, tagsStyles.
      </Text>
      <View style={styles.card}>
        <FastHtmlView
          parsedAst={parsedAst}
          baseStyle={styles.htmlBaseStyle}
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
            blockquote: {
              backgroundColor: 'rgba(124, 58, 237, 0.05)',
            } as TextStyle,
          }}
          renderers={{ CodeBlock: CustomCodeBlock }}
          onLinkPress={(url: string) => Alert.alert('onLinkPress', url)}
        />
      </View>
    </ScrollView>
  );
}

// ─── Tab 2: New Features (All 11 Optimizations) ──────────────────────────────

function NewFeaturesTab({ parsedAst }: { parsedAst: ParsedArticle | null }) {
  const [themeMode, setThemeMode] = useState<'auto' | 'light' | 'dark'>('auto');
  const [fontFeature, setFontFeature] = useState<
    'normal' | 'tnum' | 'frac' | 'smcp'
  >('tnum');
  const [asyncTime, setAsyncTime] = useState<number | null>(null);
  const [bufferSize, setBufferSize] = useState<number | null>(null);

  const handleAsyncParse = useCallback(async () => {
    const t0 = performance.now();
    const result = await parseHTMLAsync(NEW_FEATURES_HTML);
    const elapsed = performance.now() - t0;
    setAsyncTime(elapsed);
    Alert.alert(
      'Off-Thread Async Parse Succeeded',
      `Parsed in ${elapsed.toFixed(3)} ms on Nitro background thread.\nBlocks extracted: ${getBlocks(result).length}`
    );
  }, []);

  const handleExportBuffer = useCallback(() => {
    if (!parsedAst) return;
    const buf = parsedAst.toBuffer();
    setBufferSize(buf.byteLength);
    Alert.alert(
      'Binary AST Buffer Exported',
      `Serialized to zero-copy ArrayBuffer: ${buf.byteLength} bytes.`
    );
  }, [parsedAst]);

  const dynamicBaseStyle = useMemo(
    () => ({ color: themeMode === 'dark' ? '#f8fafc' : '#1e293b' }),
    [themeMode]
  );

  return (
    <ScrollView contentContainerStyle={styles.tabContent}>
      {/* Controls Card */}
      <View style={[styles.card, styles.controlCard]}>
        <Text style={styles.sectionLabel}>Live Optimizations Engine</Text>

        {/* Theme Mode Toggle */}
        <Text style={styles.controlLabel}>
          Native Dynamic Color / Dark Mode:
        </Text>
        <View style={styles.controlRow}>
          {(['auto', 'light', 'dark'] as const).map((m) => (
            <TouchableOpacity
              key={m}
              style={[
                styles.smallBtn,
                themeMode === m && styles.activeSmallBtn,
              ]}
              onPress={() => setThemeMode(m)}
            >
              <Text
                style={[
                  styles.smallBtnText,
                  themeMode === m && styles.activeSmallBtnText,
                ]}
              >
                {m.toUpperCase()}
              </Text>
            </TouchableOpacity>
          ))}
        </View>

        {/* OpenType Features Toggle */}
        <Text style={styles.controlLabel}>OpenType Typography Features:</Text>
        <View style={styles.controlRow}>
          {(['normal', 'tnum', 'frac', 'smcp'] as const).map((f) => (
            <TouchableOpacity
              key={f}
              style={[
                styles.smallBtn,
                fontFeature === f && styles.activeSmallBtn,
              ]}
              onPress={() => setFontFeature(f)}
            >
              <Text
                style={[
                  styles.smallBtnText,
                  fontFeature === f && styles.activeSmallBtnText,
                ]}
              >
                {f}
              </Text>
            </TouchableOpacity>
          ))}
        </View>

        {/* Action buttons */}
        <View style={styles.actionRow}>
          <TouchableOpacity
            style={[styles.actionBtn, styles.actionBtnPrimary]}
            onPress={handleAsyncParse}
          >
            <Text style={styles.actionBtnText}>
              ⚡ parseHTMLAsync
              {asyncTime != null ? ` (${asyncTime.toFixed(2)}ms)` : ''}
            </Text>
          </TouchableOpacity>
          <TouchableOpacity
            style={[styles.actionBtn, styles.actionBtnSecondary]}
            onPress={handleExportBuffer}
          >
            <Text style={styles.actionBtnText}>
              📦 toBuffer()
              {bufferSize != null ? ` (${bufferSize} B)` : ''}
            </Text>
          </TouchableOpacity>
        </View>
      </View>

      {/* Rendered HTML */}
      <View
        style={[styles.card, themeMode === 'dark' ? styles.cardDark : null]}
      >
        <FastHtmlView
          html={NEW_FEATURES_HTML}
          themeMode={themeMode}
          baseStyle={dynamicBaseStyle}
          fontFeatureSettings={
            fontFeature === 'normal' ? undefined : `"${fontFeature}" 1`
          }
          tagsStyles={{
            table: { marginVertical: 8 },
            th: {
              backgroundColor: themeMode === 'dark' ? '#334155' : '#e2e8f0',
            },
          }}
          onLinkPress={(url: string) => Alert.alert('Link Press', url)}
        />
      </View>
    </ScrollView>
  );
}

// ─── Tab 3: Infinite Scale (Long Document) ───────────────────────────────────

function InfiniteScaleTab() {
  return (
    <ScrollView contentContainerStyle={styles.tabContent}>
      <View style={styles.virtualHeader}>
        <Text style={styles.virtualHeaderTitle}>
          Infinite Scale FastHtmlView
        </Text>
        <Text style={styles.virtualHeaderSub}>
          40 Sections · 0 React VDOM Nodes · 100% Native Viewport Layout
        </Text>
      </View>
      <View style={styles.card}>
        <FastHtmlView
          html={LONG_HTML}
          baseStyle={styles.virtualBaseStyle}
          tagsStyles={{
            h2: { color: '#0369a1', fontWeight: '700' } as TextStyle,
            h3: { color: '#0891b2', fontWeight: '600' } as TextStyle,
            h4: { color: '#0e7490' } as TextStyle,
          }}
          onLinkPress={(url: string) => Alert.alert('Link Clicked', url)}
        />
      </View>
    </ScrollView>
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
          ? 'parseHTMLToJSON(html) — 1-pass C++ Lexbor serialization. Native memory freed instantly. Best for MMKV/SQLite caching.'
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
  const [activeTab, setActiveTab] = useState<Tab>('FastHtmlView');

  // Parse once — shared across tabs
  const parsedAst = useMemo(() => parseHTML(RICH_HTML), []);
  const blockCount = useMemo(() => getBlocks(parsedAst).length, [parsedAst]);

  return (
    <SafeAreaView style={styles.root}>
      {/* Header */}
      <View style={styles.header}>
        <Text style={styles.headerTitle}>react-native-fast-html-parser</Text>
        <Text style={styles.headerSub}>
          {blockCount} blocks · C++ (Lexbor) core · 100% Native FastHtmlView
        </Text>
      </View>

      {/* Tab bar */}
      <View style={styles.tabBarWrapper}>
        <ScrollView
          horizontal
          showsHorizontalScrollIndicator={false}
          contentContainerStyle={styles.tabBar}
        >
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
        </ScrollView>
      </View>

      {/* Active tab */}
      <View style={styles.tabBody}>
        {activeTab === 'FastHtmlView' && <RenderedTab parsedAst={parsedAst} />}
        {activeTab === 'New Features' && (
          <NewFeaturesTab parsedAst={parsedAst} />
        )}
        {activeTab === 'Infinite Scale' && <InfiniteScaleTab />}
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
  tabBarWrapper: {
    backgroundColor: '#ffffff',
    borderBottomWidth: 1,
    borderBottomColor: '#e2e8f0',
  },
  tabBar: {
    flexDirection: 'row',
  },
  tabBtn: {
    paddingVertical: 11,
    paddingHorizontal: 14,
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

  htmlBaseStyle: {
    color: '#1e293b',
    fontSize: 15,
  },
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

  // Virtualized / Infinite Scale tab
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
  virtualBaseStyle: {
    fontSize: 15,
    color: '#1e293b',
    lineHeight: 24,
  },
  controlLabel: {
    fontSize: 12,
    fontWeight: '600',
    color: '#475569',
    marginTop: 8,
    marginBottom: 4,
  },
  controlRow: {
    flexDirection: 'row',
    gap: 6,
    marginBottom: 4,
  },
  smallBtn: {
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderRadius: 6,
    backgroundColor: '#f1f5f9',
    borderWidth: 1,
    borderColor: '#cbd5e1',
  },
  activeSmallBtn: {
    backgroundColor: '#7c3aed',
    borderColor: '#7c3aed',
  },
  smallBtnText: {
    fontSize: 11,
    fontWeight: '600',
    color: '#475569',
  },
  activeSmallBtnText: {
    color: '#ffffff',
  },
  actionBtn: {
    paddingVertical: 10,
    paddingHorizontal: 12,
    borderRadius: 8,
    alignItems: 'center',
  },
  actionBtnPrimary: {
    flex: 1,
    backgroundColor: '#7c3aed',
  },
  actionBtnSecondary: {
    flex: 1,
    backgroundColor: '#0284c7',
  },
  actionBtnText: {
    color: '#ffffff',
    fontSize: 12,
    fontWeight: '700',
  },
  controlCard: {
    marginBottom: 14,
  },
  actionRow: {
    flexDirection: 'row',
    gap: 8,
    marginTop: 10,
  },
  cardDark: {
    backgroundColor: '#1e293b',
  },
});
