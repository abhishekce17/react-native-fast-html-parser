import React, { useMemo } from 'react';
import {
  View,
  Text,
  Image,
  TouchableOpacity,
  ScrollView,
  StyleSheet,
  type TextStyle,
  type ViewStyle,
  type ImageStyle,
} from 'react-native';
import { getHostComponent, callback } from 'react-native-nitro-modules';
import { parseHTML, parseHTMLAsync, estimateHtmlHeight } from '../parser';
import { getBlocks, getRows, getCells, getChildren } from '../wrappers';
import type {
  NativeHtmlViewProps,
  NativeHtmlViewMethods,
  NativeTextStyle,
} from '../NativeHtmlView.nitro';
import type { FastHtmlViewProps } from './types';
import type {
  ContentBlock,
  TableCell,
  ListItem,
  InlineNode,
  ParsedArticle,
} from '../FastHtmlParser.nitro';

export const NativeHtmlView = getHostComponent<
  NativeHtmlViewProps,
  NativeHtmlViewMethods
>('NativeHtmlView', () => ({
  uiViewClassName: 'NativeHtmlView',
  bubblingEventTypes: {},
  directEventTypes: {},
  validAttributes: {
    html: true,
    baseStyle: true,
    tagsStyles: true,
    selectable: true,
    onLinkPress: true,
    onContentSizeChange: true,
    themeMode: true,
  },
}));

function convertTextStyle(
  style?: (TextStyle & { fontFeatureSettings?: string }) | ViewStyle
): NativeTextStyle | undefined {
  if (!style) return undefined;
  const s = style as TextStyle & { fontFeatureSettings?: string };
  return {
    fontSize: typeof s.fontSize === 'number' ? s.fontSize : undefined,
    color: typeof s.color === 'string' ? s.color : undefined,
    lineHeight: typeof s.lineHeight === 'number' ? s.lineHeight : undefined,
    fontFamily: typeof s.fontFamily === 'string' ? s.fontFamily : undefined,
    fontWeight: typeof s.fontWeight === 'string' ? s.fontWeight : undefined,
    fontStyle: typeof s.fontStyle === 'string' ? s.fontStyle : undefined,
    letterSpacing:
      typeof s.letterSpacing === 'number' ? s.letterSpacing : undefined,
    textAlign: typeof s.textAlign === 'string' ? s.textAlign : undefined,
    backgroundColor:
      typeof s.backgroundColor === 'string' ? s.backgroundColor : undefined,
    fontFeatureSettings:
      typeof s.fontFeatureSettings === 'string'
        ? s.fontFeatureSettings
        : undefined,
  };
}

function convertTagsStyles(
  tagsStyles?: Record<string, TextStyle | ViewStyle>
): Record<string, NativeTextStyle> | undefined {
  if (!tagsStyles) return undefined;
  const result: Record<string, NativeTextStyle> = {};
  for (const [key, value] of Object.entries(tagsStyles)) {
    const converted = convertTextStyle(value);
    if (converted) {
      result[key] = converted;
    }
  }
  return result;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function serializeInlineNode(node: InlineNode | null | undefined): string {
  if (!node) return '';

  let inner = '';
  if (node.childCount > 0) {
    for (let i = 0; i < node.childCount; i++) {
      inner += serializeInlineNode(node.getChild(i));
    }
  } else if (node.text) {
    inner = escapeHtml(node.text);
  }

  switch (node.type) {
    case 'Bold':
      return `<b>${inner}</b>`;
    case 'Italic':
      return `<i>${inner}</i>`;
    case 'Link':
      return `<a href="${node.url}">${inner}</a>`;
    case 'InlineCode':
    case 'Code':
      return `<code>${inner}</code>`;
    case 'Break':
      return '<br />';
    default:
      return inner;
  }
}

function extractInlineText(
  node: ContentBlock | TableCell | ListItem | null | undefined
): string {
  if (!node) return '';
  let text = '';
  const count = node.childCount;
  for (let i = 0; i < count; i++) {
    const child = node.getChild(i);
    if (child) {
      text += serializeInlineNode(child);
    }
  }
  return text;
}

function serializeTableToHtml(block: ContentBlock): string {
  const rows = getRows(block);
  if (rows.length === 0) return '';
  let html = '<table>';
  rows.forEach((row, r) => {
    html += '<tr>';
    const cells = getCells(row);
    const isHeader = r === 0;
    cells.forEach((cell) => {
      const tag = isHeader ? 'th' : 'td';
      html += `<${tag}>${extractInlineText(cell)}</${tag}>`;
    });
    html += '</tr>';
  });
  html += '</table>';
  return html;
}

function serializeListItemToHtml(item: ListItem): string {
  let text = `<li>${extractInlineText(item)}`;
  const nestedCount = item.nestedCount;
  for (let i = 0; i < nestedCount; i++) {
    const nested = item.getNested(i);
    if (nested) {
      text += serializeBlockToHtml(nested);
    }
  }
  text += '</li>';
  return text;
}

function serializeListToHtml(block: ContentBlock): string {
  const isOrdered = block.ordered;
  const tag = isOrdered ? 'ol' : 'ul';
  let html = `<${tag}>`;
  const count = block.itemCount;
  for (let i = 0; i < count; i++) {
    const item = block.getItem(i);
    if (item) {
      html += serializeListItemToHtml(item);
    }
  }
  html += `</${tag}>`;
  return html;
}

function serializeQuoteToHtml(block: ContentBlock): string {
  let html = '<blockquote>';
  const count = block.quoteChildCount;
  if (count > 0) {
    for (let i = 0; i < count; i++) {
      const child = block.getQuoteChild(i);
      if (child) {
        html += serializeBlockToHtml(child);
      }
    }
  } else {
    html += `<p>${extractInlineText(block)}</p>`;
  }
  html += '</blockquote>';
  return html;
}

function serializeDefinitionListToHtml(block: ContentBlock): string {
  let html = '';
  const count = block.defItemCount;
  for (let i = 0; i < count; i++) {
    const item = block.getDefItem(i);
    if (item) {
      let termText = '';
      const termCount = item.termCount;
      for (let t = 0; t < termCount; t++) {
        const term = item.getTerm(t);
        if (term) termText += escapeHtml(term.text || '');
      }
      let defText = '';
      const defCount = item.defCount;
      for (let d = 0; d < defCount; d++) {
        const def = item.getDef(d);
        if (def) defText += escapeHtml(def.text || '');
      }
      html += `<p><b>${termText}</b><br/>${defText}</p>`;
    }
  }
  return html;
}

/**
 * Reconstructs a clean HTML snippet for a block to be rendered natively.
 */
function serializeBlockToHtml(block: ContentBlock): string {
  switch (block.type) {
    case 'Heading':
      return `<h${block.level || 1}>${extractInlineText(block)}</h${block.level || 1}>`;
    case 'Paragraph':
      return `<p>${extractInlineText(block)}</p>`;
    case 'Quote':
      return serializeQuoteToHtml(block);
    case 'CodeBlock':
      return `<pre><code${block.language ? ` class="${escapeHtml(block.language)}"` : ''}>${escapeHtml(block.code || '')}</code></pre>`;
    case 'List':
      return serializeListToHtml(block);
    case 'Table':
      return serializeTableToHtml(block);
    case 'Separator':
      return '<hr />';
    case 'Image':
      return block.linkUrl
        ? `<a href="${block.linkUrl}"><img src="${block.url}" alt="${escapeHtml(block.alt || '')}" /></a>`
        : `<img src="${block.url}" alt="${escapeHtml(block.alt || '')}" />`;
    case 'Figure':
      return `<figure><img src="${block.url}" alt="${escapeHtml(block.alt || '')}" />${block.caption ? `<figcaption>${escapeHtml(block.caption)}</figcaption>` : ''}</figure>`;
    case 'DefinitionList':
      return serializeDefinitionListToHtml(block);
    default:
      return `<p>${extractInlineText(block)}</p>`;
  }
}

interface TableRendererProps {
  block: ContentBlock;
  baseStyle?: TextStyle;
  tagsStyles?: Record<string, TextStyle | ViewStyle>;
  themeMode?: string;
  fontFeatureSettings?: string;
  onLinkPress?: (url: string) => void;
}

function renderInlineNodeToReact(
  node: InlineNode | null | undefined,
  key: string | number,
  isDark: boolean,
  onLinkPress?: (url: string) => void
): React.ReactNode {
  if (!node) return null;

  const children: React.ReactNode =
    node.childCount > 0
      ? Array.from({ length: node.childCount }, (_, idx) =>
          renderInlineNodeToReact(
            node.getChild(idx),
            `${key}-${idx}`,
            isDark,
            onLinkPress
          )
        )
      : node.text;

  switch (node.type) {
    case 'Link':
      return (
        <Text
          key={key}
          style={[
            styles.tableLinkText,
            isDark ? styles.tableLinkTextDark : styles.tableLinkTextLight,
          ]}
          onPress={onLinkPress ? () => onLinkPress(node.url) : undefined}
        >
          {children}
        </Text>
      );
    case 'Bold':
      return (
        <Text key={key} style={styles.boldText}>
          {children}
        </Text>
      );
    case 'Italic':
      return (
        <Text key={key} style={styles.italicText}>
          {children}
        </Text>
      );
    case 'InlineCode':
    case 'Code':
      return (
        <Text
          key={key}
          style={[
            styles.codeTextBase,
            isDark ? styles.codeTextDark : styles.codeTextLight,
          ]}
        >
          {children}
        </Text>
      );
    default:
      return <Text key={key}>{children}</Text>;
  }
}

function renderCellInlineChildren(
  cell: TableCell,
  isDark: boolean,
  onLinkPress?: (url: string) => void
): React.ReactNode {
  const children = getChildren(cell);
  if (children.length === 0) return null;

  return children.map((child, i) =>
    renderInlineNodeToReact(child, i, isDark, onLinkPress)
  );
}

export function DefaultTableRenderer({
  block,
  baseStyle,
  tagsStyles,
  themeMode,
  fontFeatureSettings,
  onLinkPress,
}: TableRendererProps): React.ReactElement | null {
  const rows = getRows(block);

  const maxCols = React.useMemo(() => {
    return rows.reduce((max, r) => Math.max(max, getCells(r).length), 1);
  }, [rows]);

  if (rows.length === 0) return null;

  const isDark = themeMode === 'dark';
  const tableCustomStyle = tagsStyles?.table as ViewStyle | undefined;
  const thCustomStyle = tagsStyles?.th as TextStyle | undefined;
  const tdCustomStyle = tagsStyles?.td as TextStyle | undefined;
  const trCustomStyle = tagsStyles?.tr as ViewStyle | undefined;

  const isFluidTable = maxCols <= 3;
  const cellWidthStyle: ViewStyle = isFluidTable
    ? { width: `${100 / maxCols}%` as any, flexGrow: 0, flexShrink: 0 }
    : { minWidth: 110, width: 110, flexGrow: 0, flexShrink: 0 };

  return (
    <ScrollView
      horizontal={!isFluidTable}
      showsHorizontalScrollIndicator={!isFluidTable}
      style={styles.tableScrollView}
      contentContainerStyle={[
        styles.tableScrollContent,
        isFluidTable ? styles.tableFluidContent : null,
      ]}
      nestedScrollEnabled={true}
    >
      <View
        style={[
          styles.tableContainer,
          isFluidTable ? styles.tableFluidContent : null,
          isDark ? styles.tableContainerDark : styles.tableContainerLight,
          tableCustomStyle,
        ]}
      >
        {rows.map((row, rowIndex) => {
          const isHeader = rowIndex === 0;
          const cells = getCells(row);
          const isEven = rowIndex % 2 === 0;

          return (
            <View
              key={`row-${rowIndex}`}
              style={[
                styles.tableRow,
                isHeader
                  ? isDark
                    ? styles.tableHeaderRowDark
                    : styles.tableHeaderRowLight
                  : isEven
                    ? isDark
                      ? styles.tableRowDarkEven
                      : styles.tableRowLightEven
                    : null,
                rowIndex === rows.length - 1 && styles.tableLastRow,
                trCustomStyle,
              ]}
            >
              {cells.map((cell, cellIndex) => {
                return (
                  <View
                    key={`cell-${rowIndex}-${cellIndex}`}
                    style={[
                      styles.tableCell,
                      cellWidthStyle,
                      isHeader ? styles.tableHeaderCell : styles.tableDataCell,
                      cellIndex === cells.length - 1 && styles.tableLastCell,
                      isDark
                        ? styles.tableCellBorderDark
                        : styles.tableCellBorderLight,
                      isHeader && (thCustomStyle as ViewStyle),
                      !isHeader && (tdCustomStyle as ViewStyle),
                    ]}
                  >
                    <Text
                      style={[
                        styles.tableCellText,
                        isHeader
                          ? isDark
                            ? styles.tableHeaderTextDark
                            : styles.tableHeaderTextLight
                          : isDark
                            ? styles.tableDataTextDark
                            : styles.tableDataTextLight,
                        baseStyle,
                        fontFeatureSettings
                          ? ({ fontFeatureSettings } as TextStyle)
                          : null,
                        isHeader ? thCustomStyle : tdCustomStyle,
                      ]}
                      numberOfLines={1}
                    >
                      {renderCellInlineChildren(cell, isDark, onLinkPress)}
                    </Text>
                  </View>
                );
              })}
            </View>
          );
        })}
      </View>
    </ScrollView>
  );
}

export interface ImageRendererProps {
  block: ContentBlock;
  baseStyle?: TextStyle;
  tagsStyles?: Record<string, TextStyle | ViewStyle | ImageStyle>;
  themeMode?: string;
  onLinkPress?: (url: string) => void;
}

export function DefaultImageRenderer({
  block,
  tagsStyles,
  onLinkPress,
}: ImageRendererProps): React.ReactElement | null {
  const [aspectRatio, setAspectRatio] = React.useState<number>(16 / 9);

  React.useEffect(() => {
    if (block.url) {
      Image.getSize(
        block.url,
        (w, h) => {
          if (w > 0 && h > 0) {
            setAspectRatio(w / h);
          }
        },
        () => {
          // Keep default 16/9 if size cannot be determined
        }
      );
    }
  }, [block.url]);

  const imgCustomStyle = tagsStyles?.img as ImageStyle | undefined;

  const imageElement = (
    <Image
      source={{ uri: block.url }}
      accessibilityLabel={block.alt || 'Image'}
      style={[styles.defaultImage, { aspectRatio }, imgCustomStyle]}
      resizeMode="cover"
    />
  );

  if (block.linkUrl) {
    return (
      <TouchableOpacity
        activeOpacity={0.85}
        onPress={() => onLinkPress?.(block.linkUrl)}
        style={styles.imageWrapper}
      >
        {imageElement}
      </TouchableOpacity>
    );
  }

  return <View style={styles.imageWrapper}>{imageElement}</View>;
}

export function DefaultFigureRenderer({
  block,
  baseStyle,
  tagsStyles,
  themeMode,
  onLinkPress,
}: ImageRendererProps): React.ReactElement | null {
  const isDark = themeMode === 'dark';
  const figcaptionStyle = tagsStyles?.figcaption as TextStyle | undefined;
  const figureStyle = tagsStyles?.figure as ViewStyle | undefined;

  return (
    <View style={[styles.figureContainer, figureStyle]}>
      <DefaultImageRenderer
        block={block}
        tagsStyles={tagsStyles}
        onLinkPress={onLinkPress}
      />
      {block.caption ? (
        <Text
          style={[
            styles.figcaptionText,
            isDark ? styles.figcaptionTextDark : styles.figcaptionTextLight,
            baseStyle,
            figcaptionStyle,
          ]}
        >
          {block.caption}
        </Text>
      ) : null}
    </View>
  );
}

interface AutoSizingViewProps {
  html: string;
  baseStyle?: NativeTextStyle;
  tagsStyles?: Record<string, NativeTextStyle>;
  selectable?: boolean;
  themeMode?: string;
  onLinkPress?: (url: string) => void;
  style?: ViewStyle | (ViewStyle | undefined)[];
}

// Bounded in-memory measured height cache (LRU eviction) to eliminate native measurement delays on screen revisits
const MEASURED_HEIGHT_CACHE = new Map<string, number>();
const MAX_HEIGHT_CACHE_SIZE = 500;

function cacheMeasuredHeight(key: string, height: number) {
  if (MEASURED_HEIGHT_CACHE.size >= MAX_HEIGHT_CACHE_SIZE) {
    const firstKey = MEASURED_HEIGHT_CACHE.keys().next().value;
    if (firstKey) MEASURED_HEIGHT_CACHE.delete(firstKey);
  }
  MEASURED_HEIGHT_CACHE.set(key, height);
}

function AutoSizingNativeHtmlView({
  html,
  baseStyle,
  tagsStyles,
  selectable,
  themeMode,
  onLinkPress,
  style,
}: AutoSizingViewProps) {
  const cacheKey = React.useMemo(
    () => `${baseStyle?.fontSize ?? 16}_${baseStyle?.lineHeight ?? 22}_${html}`,
    [html, baseStyle?.fontSize, baseStyle?.lineHeight]
  );

  const initialEstimate = React.useMemo(() => {
    const cached = MEASURED_HEIGHT_CACHE.get(cacheKey);
    if (cached !== undefined && cached > 0) return cached;
    return estimateHtmlHeight(html, baseStyle?.lineHeight ?? 22);
  }, [html, cacheKey, baseStyle?.lineHeight]);

  const [measuredHeight, setMeasuredHeight] = React.useState<
    number | undefined
  >(() => MEASURED_HEIGHT_CACHE.get(cacheKey));

  const handleSizeChange = React.useCallback(
    (h: number) => {
      if (h > 0) {
        cacheMeasuredHeight(cacheKey, h);
        setMeasuredHeight(h);
      }
    },
    [cacheKey]
  );

  const wrappedSizeChange = React.useMemo(
    () => callback(handleSizeChange),
    [handleSizeChange]
  );

  const wrappedOnLinkPress = React.useMemo(
    () => (onLinkPress ? callback(onLinkPress) : undefined),
    [onLinkPress]
  );

  const activeHeight = measuredHeight ?? initialEstimate;

  return (
    <NativeHtmlView
      html={html}
      baseStyle={baseStyle}
      tagsStyles={tagsStyles}
      selectable={selectable}
      themeMode={themeMode}
      onLinkPress={wrappedOnLinkPress}
      onContentSizeChange={wrappedSizeChange}
      style={[
        styles.container,
        activeHeight > 0 ? { height: activeHeight } : null,
        style,
      ]}
    />
  );
}

export function FastHtmlView({
  html,
  mode = 'sync',
  parsedAst,
  baseStyle,
  tagsStyles,
  renderers = {},
  selectable = true,
  themeMode,
  fontFeatureSettings,
  onLinkPress,
  style,
}: FastHtmlViewProps): React.ReactElement | null {
  const nativeBaseStyle = useMemo(() => {
    const converted = convertTextStyle(baseStyle) || {};
    if (fontFeatureSettings && !converted.fontFeatureSettings) {
      converted.fontFeatureSettings = fontFeatureSettings;
    }
    return Object.keys(converted).length > 0 ? converted : undefined;
  }, [baseStyle, fontFeatureSettings]);

  const nativeTagsStyles = useMemo(
    () => convertTagsStyles(tagsStyles),
    [tagsStyles]
  );

  const [article, setArticle] = React.useState<ParsedArticle | null>(() => {
    if (parsedAst) return parsedAst;
    if (html && mode === 'sync') {
      try {
        return parseHTML(html);
      } catch {
        return null;
      }
    }
    return null;
  });

  React.useEffect(() => {
    if (parsedAst) {
      setArticle(parsedAst);
      return;
    }
    if (html) {
      if (mode === 'async') {
        let isMounted = true;
        parseHTMLAsync(html)
          .then((parsed) => {
            if (isMounted) {
              setArticle(parsed);
            }
          })
          .catch(() => {
            if (isMounted) {
              setArticle(null);
            }
          });
        return () => {
          isMounted = false;
        };
      } else {
        try {
          const parsed = parseHTML(html);
          setArticle(parsed);
        } catch {
          setArticle(null);
        }
      }
    } else {
      setArticle(null);
    }
    return undefined;
  }, [html, mode, parsedAst]);

  const blocks = useMemo(() => getBlocks(article), [article]);

  const hasSpecialBlocks = useMemo(() => {
    if (Object.keys(renderers).length > 0) return true;
    return blocks.some(
      (b) =>
        b.type === 'Table' ||
        b.type === 'Figure' ||
        b.type === 'Image' ||
        b.type === 'Video' ||
        b.type === 'Audio'
    );
  }, [renderers, blocks]);

  // Fast Path: 100% pure native execution when no custom renderers or special blocks are present
  if (!hasSpecialBlocks) {
    const rawHtml = html || (article ? article.toJSON() : '');
    return (
      <AutoSizingNativeHtmlView
        html={rawHtml}
        baseStyle={nativeBaseStyle}
        tagsStyles={nativeTagsStyles}
        selectable={selectable}
        themeMode={themeMode}
        onLinkPress={onLinkPress}
        style={[styles.container, style]}
      />
    );
  }

  if (!article) return null;

  // Segmented Block Stream Engine: seamlessly embeds custom React renderers, tables & media between native text chunks
  const segments: React.ReactNode[] = [];
  let pendingBlockHtml = '';
  let segmentKey = 0;

  const flushPendingText = () => {
    if (pendingBlockHtml.trim().length > 0) {
      segments.push(
        <AutoSizingNativeHtmlView
          key={`native-segment-${segmentKey++}`}
          html={pendingBlockHtml}
          baseStyle={nativeBaseStyle}
          tagsStyles={nativeTagsStyles}
          selectable={selectable}
          themeMode={themeMode}
          onLinkPress={onLinkPress}
          style={styles.textSegment}
        />
      );
      pendingBlockHtml = '';
    }
  };

  blocks.forEach((block, index) => {
    const CustomRenderer =
      renderers[block.type] || renderers[block.type.toLowerCase()];

    if (CustomRenderer) {
      flushPendingText();
      segments.push(
        <CustomRenderer
          key={`custom-block-${index}`}
          block={block}
          baseStyle={baseStyle}
        />
      );
    } else if (block.type === 'Table') {
      flushPendingText();
      segments.push(
        <DefaultTableRenderer
          key={`table-block-${index}`}
          block={block}
          baseStyle={baseStyle}
          tagsStyles={tagsStyles}
          themeMode={themeMode}
          fontFeatureSettings={fontFeatureSettings}
          onLinkPress={onLinkPress}
        />
      );
    } else if (block.type === 'Figure') {
      flushPendingText();
      segments.push(
        <DefaultFigureRenderer
          key={`figure-block-${index}`}
          block={block}
          baseStyle={baseStyle}
          tagsStyles={tagsStyles}
          themeMode={themeMode}
          onLinkPress={onLinkPress}
        />
      );
    } else if (block.type === 'Image') {
      flushPendingText();
      segments.push(
        <DefaultImageRenderer
          key={`image-block-${index}`}
          block={block}
          baseStyle={baseStyle}
          tagsStyles={tagsStyles}
          themeMode={themeMode}
          onLinkPress={onLinkPress}
        />
      );
    } else {
      pendingBlockHtml += serializeBlockToHtml(block);
    }
  });

  flushPendingText();

  return <View style={[styles.container, style]}>{segments}</View>;
}

const styles = StyleSheet.create({
  container: {
    width: '100%',
  },
  textSegment: {
    width: '100%',
  },
  tableScrollView: {
    width: '100%',
    marginVertical: 10,
  },
  tableScrollContent: {
    minWidth: '100%',
    paddingVertical: 2,
  },
  tableContainer: {
    minWidth: '100%',
    borderWidth: 1,
    borderRadius: 8,
    overflow: 'hidden',
  },
  tableContainerLight: {
    borderColor: '#e2e8f0',
    backgroundColor: '#ffffff',
  },
  tableContainerDark: {
    borderColor: '#334155',
    backgroundColor: '#0f172a',
  },
  tableRow: {
    flexDirection: 'row',
    alignItems: 'stretch',
    borderBottomWidth: 1,
    minWidth: '100%',
  },
  tableLastRow: {
    borderBottomWidth: 0,
  },
  tableHeaderRowLight: {
    backgroundColor: '#f1f5f9',
    borderBottomColor: '#cbd5e1',
  },
  tableHeaderRowDark: {
    backgroundColor: '#1e293b',
    borderBottomColor: '#475569',
  },
  tableRowLightEven: {
    backgroundColor: '#f8fafc',
    borderBottomColor: '#e2e8f0',
  },
  tableRowDarkEven: {
    backgroundColor: '#182234',
    borderBottomColor: '#334155',
  },
  tableCell: {
    paddingHorizontal: 16,
    paddingVertical: 10,
    borderRightWidth: 1,
    justifyContent: 'center',
  },
  tableHeaderCell: {
    paddingVertical: 11,
  },
  tableDataCell: {
    paddingVertical: 9,
  },
  tableLastCell: {
    borderRightWidth: 0,
  },
  tableCellBorderLight: {
    borderRightColor: '#e2e8f0',
  },
  tableCellBorderDark: {
    borderRightColor: '#334155',
  },
  tableCellText: {
    fontSize: 13,
    lineHeight: 18,
  },
  tableHeaderTextLight: {
    color: '#0f172a',
    fontWeight: '700',
  },
  tableHeaderTextDark: {
    color: '#f8fafc',
    fontWeight: '700',
  },
  tableDataTextLight: {
    color: '#334155',
  },
  tableDataTextDark: {
    color: '#cbd5e1',
  },
  tableLinkText: {
    textDecorationLine: 'underline',
  },
  imageWrapper: {
    width: '100%',
    marginVertical: 6,
    borderRadius: 8,
    overflow: 'hidden',
  },
  defaultImage: {
    width: '100%',
    borderRadius: 8,
    backgroundColor: 'rgba(128,128,128,0.08)',
  },
  figureContainer: {
    width: '100%',
    marginVertical: 10,
  },
  tableFluidContent: {
    minWidth: '100%',
    width: '100%',
  },
  boldText: {
    fontWeight: '700',
  },
  italicText: {
    fontStyle: 'italic',
  },
  codeTextBase: {
    fontFamily: 'Courier',
    paddingHorizontal: 3,
    borderRadius: 3,
  },
  codeTextDark: {
    backgroundColor: 'rgba(255,255,255,0.1)',
  },
  codeTextLight: {
    backgroundColor: 'rgba(0,0,0,0.06)',
  },
  tableLinkTextDark: {
    color: '#60a5fa',
  },
  tableLinkTextLight: {
    color: '#2563eb',
  },
  figcaptionText: {
    marginTop: 6,
    fontSize: 13,
    lineHeight: 18,
    fontStyle: 'italic',
    textAlign: 'center',
    paddingHorizontal: 8,
  },
  figcaptionTextDark: {
    color: '#94a3b8',
  },
  figcaptionTextLight: {
    color: '#64748b',
  },
});
