import React, { useMemo, useCallback } from 'react';
import { FlatList, StyleSheet, type ListRenderItemInfo } from 'react-native';
import { parseHTML } from '../parser';
import { getBlocks } from '../wrappers';
import { HtmlRenderer } from './HtmlRenderer';
import type { ContentBlock } from '../FastHtmlParser.nitro';
import type { VirtualizedHtmlRendererProps } from './types';

export function VirtualizedHtmlRenderer({
  html,
  parsedAst,
  baseStyle,
  tagsStyles,
  renderers,
  inlineRenderers,
  onLinkPress,
  ListHeaderComponent,
  ListFooterComponent,
  contentContainerStyle,
  style,
}: VirtualizedHtmlRendererProps): React.ReactElement | null {
  const article = useMemo(() => {
    if (parsedAst) return parsedAst;
    if (html) return parseHTML(html);
    return null;
  }, [html, parsedAst]);

  const blocks = useMemo(() => getBlocks(article), [article]);

  const renderItem = useCallback(
    ({ item, index }: ListRenderItemInfo<ContentBlock>) => {
      // Create a virtual 1-block article for this row to reuse HtmlRenderer
      const singleBlockArticle = {
        length: 1,
        getBlock: (i: number) => (i === 0 ? item : null),
        toJSON: () => JSON.stringify([item]),
        equals: () => false,
        dispose: () => {},
      } as any;

      return (
        <HtmlRenderer
          key={index}
          parsedAst={singleBlockArticle}
          baseStyle={baseStyle}
          tagsStyles={tagsStyles}
          renderers={renderers}
          inlineRenderers={inlineRenderers}
          onLinkPress={onLinkPress}
        />
      );
    },
    [baseStyle, tagsStyles, renderers, inlineRenderers, onLinkPress]
  );

  const keyExtractor = useCallback(
    (_: ContentBlock, index: number) => `block-${index}`,
    []
  );

  if (!article) return null;

  return (
    <FlatList
      data={blocks}
      renderItem={renderItem}
      keyExtractor={keyExtractor}
      ListHeaderComponent={ListHeaderComponent ?? undefined}
      ListFooterComponent={ListFooterComponent ?? undefined}
      contentContainerStyle={[styles.contentContainer, contentContainerStyle]}
      style={style}
      showsVerticalScrollIndicator={false}
    />
  );
}

const styles = StyleSheet.create({
  contentContainer: {
    paddingVertical: 12,
  },
});
