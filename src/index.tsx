export { parseHTML, parseHTMLToJSON } from './parser';

export { HtmlRenderer } from './renderer/HtmlRenderer';
export { VirtualizedHtmlRenderer } from './renderer/VirtualizedHtmlRenderer';
export {
  createCanonicalAdapter,
  type CanonicalAdapterConfig,
} from './adapters';
export * from './renderer/types';

export {
  getBlocks,
  getChildren,
  getItems,
  getNestedBlocks,
  getRows,
  getCells,
  getQuoteChildren,
  getDefItems,
} from './wrappers';

export type {
  InlineNode,
  ListItem,
  TableCell,
  TableRow,
  DefinitionItem,
  ContentBlock,
  ParsedArticle,
  FastHtmlParser,
} from './FastHtmlParser.nitro';
