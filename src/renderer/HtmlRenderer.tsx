import React, { useMemo } from 'react';
import {
  View,
  Text,
  Image,
  ScrollView,
  StyleSheet,
  Linking,
  TouchableOpacity,
  type ImageStyle,
} from 'react-native';
import { parseHTML } from '../parser';
import {
  getBlocks,
  getChildren,
  getItems,
  getRows,
  getCells,
  getQuoteChildren,
  getNestedBlocks,
  getDefItems,
} from '../wrappers';
import type {
  ContentBlock,
  InlineNode,
  ListItem,
  TableRow,
  TableCell,
  DefinitionItem,
} from '../FastHtmlParser.nitro';
import type { HtmlRendererProps } from './types';

export function HtmlRenderer({
  html,
  parsedAst,
  baseStyle,
  tagsStyles = {},
  renderers = {},
  inlineRenderers = {},
  onLinkPress,
  style,
}: HtmlRendererProps): React.ReactElement | null {
  const article = useMemo(() => {
    if (parsedAst) return parsedAst;
    if (html) return parseHTML(html);
    return null;
  }, [html, parsedAst]);

  if (!article) return null;

  const handleLinkPress = (url: string) => {
    if (onLinkPress) {
      onLinkPress(url);
    } else if (url) {
      Linking.openURL(url).catch((err) => {
        console.warn('[HtmlRenderer] Failed to open URL:', url, err);
      });
    }
  };

  const renderInlineNode = (
    node: InlineNode,
    index: number
  ): React.ReactElement => {
    const CustomRenderer = inlineRenderers[node.type];
    const defaultRender = () => {
      switch (node.type) {
        case 'Bold': {
          const children = getChildren(node);
          return (
            <Text
              key={index}
              style={[styles.bold, tagsStyles.b, tagsStyles.strong]}
            >
              {children.length > 0
                ? children.map((c, idx) => renderInlineNode(c, idx))
                : node.text}
            </Text>
          );
        }
        case 'Italic': {
          const children = getChildren(node);
          return (
            <Text
              key={index}
              style={[styles.italic, tagsStyles.i, tagsStyles.em]}
            >
              {children.length > 0
                ? children.map((c, idx) => renderInlineNode(c, idx))
                : node.text}
            </Text>
          );
        }
        case 'Link': {
          const children = getChildren(node);
          return (
            <Text
              key={index}
              style={[styles.link, tagsStyles.a]}
              accessibilityRole="link"
              accessibilityHint={node.url}
              onPress={() => handleLinkPress(node.url)}
            >
              {children.length > 0
                ? children.map((c, idx) => renderInlineNode(c, idx))
                : node.text}
            </Text>
          );
        }
        case 'InlineCode': {
          return (
            <Text key={index} style={[styles.inlineCode, tagsStyles.code]}>
              {node.text}
            </Text>
          );
        }
        case 'Break': {
          return <Text key={index}>{'\n'}</Text>;
        }
        case 'Text':
        default: {
          return <Text key={index}>{node.text}</Text>;
        }
      }
    };

    if (CustomRenderer) {
      return (
        <CustomRenderer
          key={index}
          node={node}
          defaultRender={defaultRender}
          baseStyle={baseStyle}
        />
      );
    }

    return defaultRender();
  };

  const renderListItem = (
    item: ListItem,
    itemIndex: number,
    ordered: boolean
  ): React.ReactElement => {
    const inlineChildren = getChildren(item);
    const nestedBlocks = getNestedBlocks(item);

    return (
      <View key={itemIndex} style={styles.listItemRow}>
        <Text
          style={[
            styles.bullet,
            baseStyle ? { color: baseStyle.color } : null,
            tagsStyles.li,
          ]}
        >
          {ordered ? `${itemIndex + 1}. ` : '• '}
        </Text>
        <View style={styles.listItemContent}>
          <Text style={[styles.paragraphText, baseStyle, tagsStyles.p, tagsStyles.li]}>
            {inlineChildren.map((c, idx) => renderInlineNode(c, idx))}
          </Text>
          {nestedBlocks.map((nested, nIdx) => renderContentBlock(nested, nIdx))}
        </View>
      </View>
    );
  };

  const renderContentBlock = (
    block: ContentBlock,
    index: number
  ): React.ReactElement | null => {
    const CustomRenderer = renderers[block.type];

    const defaultRender = (): React.ReactElement | null => {
      switch (block.type) {
        case 'Heading': {
          const children = getChildren(block);
          const level = block.level || 1;
          const headingTag = `h${Math.min(Math.max(level, 1), 6)}`;
          const headingStyle =
            level === 1
              ? styles.h1
              : level === 2
                ? styles.h2
                : level === 3
                  ? styles.h3
                  : level === 4
                    ? styles.h4
                    : level === 5
                      ? styles.h5
                      : styles.h6;

          return (
            <Text
              key={index}
              style={[
                styles.heading,
                baseStyle,
                headingStyle,
                tagsStyles.h,
                tagsStyles[headingTag],
              ]}
              accessibilityRole="header"
              aria-level={level}
            >
              {children.map((c, idx) => renderInlineNode(c, idx))}
            </Text>
          );
        }

        case 'Paragraph': {
          const children = getChildren(block);
          if (children.length === 0) return null;
          return (
            <Text
              key={index}
              style={[styles.paragraphText, baseStyle, tagsStyles.p]}
            >
              {children.map((c, idx) => renderInlineNode(c, idx))}
            </Text>
          );
        }

        case 'CodeBlock': {
          return (
            <View
              key={index}
              style={[styles.codeBlockContainer, tagsStyles.pre]}
            >
              {block.language ? (
                <Text style={styles.codeLanguage}>{block.language}</Text>
              ) : null}
              <Text style={[styles.codeText, tagsStyles.code]}>
                {block.code}
              </Text>
            </View>
          );
        }

        case 'List': {
          const items = getItems(block);
          const isOrdered = block.ordered;
          return (
            <View
              key={index}
              style={[
                styles.listContainer,
                isOrdered ? tagsStyles.ol : tagsStyles.ul,
              ]}
            >
              {items.map((item, idx) => renderListItem(item, idx, isOrdered))}
            </View>
          );
        }

        case 'Table': {
          const rows = getRows(block);
          return (
            <ScrollView
              key={index}
              horizontal
              showsHorizontalScrollIndicator={false}
              style={styles.tableScrollView}
            >
              <View style={[styles.tableContainer, tagsStyles.table]}>
                {rows.map((row: TableRow, rIdx: number) => {
                  const cells = getCells(row);
                  return (
                    <View key={rIdx} style={[styles.tableRow, tagsStyles.tr]}>
                      {cells.map((cell: TableCell, cIdx: number) => {
                        const cellChildren = getChildren(cell);
                        return (
                          <View
                            key={cIdx}
                            style={[
                              styles.tableCell,
                              rIdx === 0 && styles.tableHeaderCell,
                              rIdx === 0 ? tagsStyles.th : tagsStyles.td,
                            ]}
                          >
                            <Text
                              style={[
                                styles.tableCellText,
                                rIdx === 0 && styles.tableHeaderCellText,
                                baseStyle,
                                rIdx === 0 ? tagsStyles.th : tagsStyles.td,
                              ]}
                            >
                              {cellChildren.map((c, idx) =>
                                renderInlineNode(c, idx)
                              )}
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

        case 'Quote': {
          const quoteChildren = getQuoteChildren(block);
          return (
            <View
              key={index}
              style={[styles.quoteContainer, tagsStyles.blockquote]}
            >
              {quoteChildren.map((childBlock, idx) =>
                renderContentBlock(childBlock, idx)
              )}
            </View>
          );
        }

        case 'Image':
        case 'Figure': {
          const imageEl = block.url ? (
            <Image
              source={{ uri: block.url }}
              style={[styles.image, tagsStyles.img as ImageStyle]}
              resizeMode="contain"
              accessibilityLabel={block.alt || 'Image'}
            />
          ) : null;

          return (
            <View
              key={index}
              style={[styles.figureContainer, tagsStyles.figure]}
            >
              {block.linkUrl && imageEl ? (
                <TouchableOpacity
                  activeOpacity={0.8}
                  onPress={() => handleLinkPress(block.linkUrl)}
                  accessibilityRole="link"
                  accessibilityHint={block.linkUrl}
                >
                  {imageEl}
                </TouchableOpacity>
              ) : (
                imageEl
              )}
              {block.caption ? (
                <Text
                  style={[
                    styles.caption,
                    baseStyle ? { color: baseStyle.color } : null,
                    tagsStyles.figcaption,
                  ]}
                >
                  {block.caption}
                </Text>
              ) : null}
            </View>
          );
        }

        case 'DefinitionList': {
          const defItems = getDefItems(block);
          return (
            <View key={index} style={[styles.defListContainer, tagsStyles.dl]}>
              {defItems.map((item: DefinitionItem, dIdx: number) => {
                const termNodes: InlineNode[] = [];
                for (let t = 0; t < item.termCount; t++) {
                  const n = item.getTerm(t);
                  if (n) termNodes.push(n);
                }
                const defNodes: InlineNode[] = [];
                for (let d = 0; d < item.defCount; d++) {
                  const n = item.getDef(d);
                  if (n) defNodes.push(n);
                }
                return (
                  <View key={dIdx} style={styles.defItem}>
                    <Text style={[styles.defTerm, baseStyle, tagsStyles.dt]}>
                      {termNodes.map((tn, idx) => renderInlineNode(tn, idx))}
                    </Text>
                    <Text style={[styles.defDesc, baseStyle, tagsStyles.dd]}>
                      {defNodes.map((dn, idx) => renderInlineNode(dn, idx))}
                    </Text>
                  </View>
                );
              })}
            </View>
          );
        }

        case 'Separator': {
          return <View key={index} style={[styles.separator, tagsStyles.hr]} />;
        }

        default:
          return null;
      }
    };

    if (CustomRenderer) {
      return (
        <CustomRenderer
          key={index}
          block={block}
          defaultRender={defaultRender}
          baseStyle={baseStyle}
        />
      );
    }

    return defaultRender();
  };

  const blocks = getBlocks(article);

  return (
    <View style={[styles.container, style]}>
      {blocks.map((block, index) => renderContentBlock(block, index))}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    width: '100%',
  },
  paragraphText: {
    fontSize: 16,
    lineHeight: 24,
    marginVertical: 6,
  },
  heading: {
    fontWeight: 'bold',
    marginVertical: 8,
  },
  h1: { fontSize: 28, lineHeight: 34 },
  h2: { fontSize: 24, lineHeight: 30 },
  h3: { fontSize: 20, lineHeight: 26 },
  h4: { fontSize: 18, lineHeight: 24 },
  h5: { fontSize: 16, lineHeight: 22 },
  h6: { fontSize: 14, lineHeight: 20 },
  bold: {
    fontWeight: 'bold',
  },
  italic: {
    fontStyle: 'italic',
  },
  link: {
    color: '#2563eb',
    textDecorationLine: 'underline',
  },
  inlineCode: {
    fontFamily: 'Courier',
    backgroundColor: 'rgba(148, 163, 184, 0.15)',
    fontSize: 14,
    paddingHorizontal: 4,
    borderRadius: 4,
  },
  codeBlockContainer: {
    backgroundColor: '#0f172a',
    borderRadius: 8,
    padding: 14,
    marginVertical: 10,
  },
  codeLanguage: {
    fontSize: 11,
    color: '#94a3b8',
    textTransform: 'uppercase',
    marginBottom: 6,
    fontWeight: '700',
  },
  codeText: {
    fontFamily: 'Courier',
    color: '#f8fafc',
    fontSize: 14,
    lineHeight: 20,
  },
  listContainer: {
    marginVertical: 6,
    paddingLeft: 4,
  },
  listItemRow: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    marginVertical: 3,
  },
  bullet: {
    fontSize: 16,
    lineHeight: 24,
    width: 20,
    opacity: 0.8,
  },
  listItemContent: {
    flex: 1,
  },
  tableScrollView: {
    marginVertical: 10,
    width: '100%',
  },
  tableContainer: {
    borderWidth: 1,
    borderColor: 'rgba(148, 163, 184, 0.25)',
    borderRadius: 6,
    overflow: 'hidden',
    minWidth: '100%',
  },
  tableRow: {
    flexDirection: 'row',
    borderBottomWidth: 1,
    borderBottomColor: 'rgba(148, 163, 184, 0.25)',
  },
  tableCell: {
    flex: 1,
    padding: 8,
  },
  tableHeaderCell: {
    backgroundColor: 'rgba(148, 163, 184, 0.1)',
  },
  tableCellText: {
    fontSize: 14,
  },
  tableHeaderCellText: {
    fontWeight: 'bold',
  },
  quoteContainer: {
    borderLeftWidth: 4,
    borderLeftColor: '#94a3b8',
    paddingLeft: 12,
    marginVertical: 8,
    backgroundColor: 'rgba(148, 163, 184, 0.08)',
    paddingVertical: 6,
    borderRadius: 2,
  },
  figureContainer: {
    marginVertical: 10,
    alignItems: 'center',
  },
  image: {
    width: '100%',
    height: 220,
    borderRadius: 8,
  },
  caption: {
    fontSize: 13,
    fontStyle: 'italic',
    marginTop: 4,
    textAlign: 'center',
    opacity: 0.75,
  },
  defListContainer: {
    marginVertical: 8,
  },
  defItem: {
    marginVertical: 4,
  },
  defTerm: {
    fontWeight: 'bold',
    fontSize: 15,
  },
  defDesc: {
    fontSize: 15,
    paddingLeft: 12,
    marginTop: 2,
    opacity: 0.85,
  },
  separator: {
    height: 1,
    backgroundColor: 'rgba(148, 163, 184, 0.25)',
    marginVertical: 14,
  },
});
