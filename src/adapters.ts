import type {
  ParsedArticleData,
  ContentBlockData,
  HeadingBlockData,
  ParagraphBlockData,
  ListBlockData,
  TableBlockData,
  ImageBlockData,
  CodeBlockData,
  QuoteBlockData,
} from './renderer/types';

/**
 * Represents a generic canonical block transformer function.
 */
export type BlockTransformer<TOutput = any> = (
  block: ContentBlockData,
  index: number
) => TOutput;

/**
 * Configuration options for creating an application-owned canonical AST adapter.
 */
export interface CanonicalAdapterConfig<
  TDomainArticle = any,
  TDomainBlock = any,
> {
  /**
   * Transforms a single parsed content block into your application's domain block model.
   */
  transformBlock?: (block: ContentBlockData, index: number) => TDomainBlock;

  /**
   * Optional custom transformers keyed by block type.
   */
  transformers?: {
    heading?: (block: HeadingBlockData, index: number) => TDomainBlock;
    paragraph?: (block: ParagraphBlockData, index: number) => TDomainBlock;
    list?: (block: ListBlockData, index: number) => TDomainBlock;
    table?: (block: TableBlockData, index: number) => TDomainBlock;
    image?: (block: ImageBlockData, index: number) => TDomainBlock;
    code?: (block: CodeBlockData, index: number) => TDomainBlock;
    quote?: (block: QuoteBlockData, index: number) => TDomainBlock;
    [key: string]: ((block: any, index: number) => TDomainBlock) | undefined;
  };

  /**
   * Transforms the final array of domain blocks and article metadata into your application's canonical article model.
   */
  transformArticle?: (
    article: ParsedArticleData,
    domainBlocks: TDomainBlock[]
  ) => TDomainArticle;
}

/**
 * Creates an application-owned canonical AST adapter that maps parser output
 * to your application's proprietary domain schema (e.g. Custom Article Schema).
 *
 * @example
 * ```typescript
 * const articleAdapter = createCanonicalAdapter({
 *   transformBlock: (block) => ({
 *     id: block.id ?? `block-${Math.random()}`,
 *     type: block.type,
 *     raw: block,
 *   }),
 *   transformArticle: (article, blocks) => ({
 *     articleId: '123',
 *     title: article.title,
 *     sections: blocks,
 *   }),
 * });
 *
 * const canonicalDoc = articleAdapter.adapt(parsedJsonArticle);
 * ```
 */
export function createCanonicalAdapter<
  TDomainArticle = any,
  TDomainBlock = any,
>(config: CanonicalAdapterConfig<TDomainArticle, TDomainBlock>) {
  return {
    /**
     * Adapt a parsed article into your application's canonical data model.
     */
    adapt(article: ParsedArticleData): TDomainArticle {
      const domainBlocks: TDomainBlock[] = (article.blocks || []).map(
        (block: ContentBlockData, index: number) => {
          // Check for specialized block transformer
          const specialized = config.transformers?.[block.type];
          if (specialized) {
            return specialized(block as any, index);
          }

          // Fallback to default block transformer
          if (config.transformBlock) {
            return config.transformBlock(block, index);
          }

          return block as unknown as TDomainBlock;
        }
      );

      if (config.transformArticle) {
        return config.transformArticle(article, domainBlocks);
      }

      return {
        ...article,
        blocks: domainBlocks,
      } as unknown as TDomainArticle;
    },
  };
}
