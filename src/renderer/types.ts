import type React from 'react';
import type { TextStyle, ViewStyle } from 'react-native';
import type {
  ContentBlock,
  InlineNode,
  ParsedArticle,
} from '../FastHtmlParser.nitro';
import type { NativeTextStyle } from '../NativeHtmlView.nitro';

export type { NativeTextStyle };

export type CustomBlockRenderer = React.ComponentType<{
  block: ContentBlock;
  baseStyle?: TextStyle;
}>;

export type CustomInlineRenderer = React.ComponentType<{
  node: InlineNode;
  baseStyle?: TextStyle;
}>;

export interface FastHtmlViewProps {
  /**
   * The raw HTML string to parse and render natively.
   */
  html?: string;

  /**
   * Parsing execution mode: 'sync' (default) or 'async' (dispatched to C++ worker).
   */
  mode?: 'sync' | 'async';

  /**
   * An already-parsed AST object (optional, if parsed ahead of time).
   */
  parsedAst?: ParsedArticle | null;

  /**
   * Base text style applied to all rendered inline content.
   */
  baseStyle?: TextStyle & { fontFeatureSettings?: string };

  /**
   * Custom style overrides for specific tags/blocks (e.g. `h1`, `h2`, `p`, `a`, `code`).
   */
  tagsStyles?: Record<string, TextStyle | ViewStyle>;

  /**
   * Custom component renderers for block elements (e.g., custom Video player, CodeBlock, Polls).
   */
  renderers?: {
    Paragraph?: CustomBlockRenderer;
    Heading?: CustomBlockRenderer;
    Image?: CustomBlockRenderer;
    Figure?: CustomBlockRenderer;
    CodeBlock?: CustomBlockRenderer;
    List?: CustomBlockRenderer;
    Table?: CustomBlockRenderer;
    Quote?: CustomBlockRenderer;
    DefinitionList?: CustomBlockRenderer;
    Video?: CustomBlockRenderer;
    Audio?: CustomBlockRenderer;
    Embed?: CustomBlockRenderer;
    Separator?: CustomBlockRenderer;
    [key: string]: CustomBlockRenderer | undefined;
  };

  /**
   * Enable or disable native continuous text selection. Defaults to `true`.
   */
  selectable?: boolean;

  /**
   * Force 'light' | 'dark' theme mode or leave 'auto' to adapt to system dark mode.
   */
  themeMode?: 'light' | 'dark' | 'auto';

  /**
   * OpenType font feature settings (e.g. '"tnum" 1', '"liga" 1', '"frac" 1').
   */
  fontFeatureSettings?: string;

  /**
   * Callback fired when an `<a>` link node is pressed.
   */
  onLinkPress?: (url: string) => void;

  /**
   * Style for the outer container.
   */
  style?: ViewStyle;
}

// ── JSON AST Schema Contracts ───────────────────────────────────────────────

export interface InlineNodeData {
  type: string;
  text?: string;
  url?: string;
  isBold?: boolean;
  isItalic?: boolean;
  attributes?: Record<string, string>;
  children?: InlineNodeData[];
}

export interface BaseBlockData {
  type: string;
  tag?: string;
  id?: string;
  className?: string;
  attributes?: Record<string, string>;
}

export interface HeadingBlockData extends BaseBlockData {
  type: 'heading';
  level: number;
  children: InlineNodeData[];
}

export interface ParagraphBlockData extends BaseBlockData {
  type: 'paragraph';
  children: InlineNodeData[];
}

export interface ListItemData {
  children: InlineNodeData[];
  nestedBlocks?: ContentBlockData[];
}

export interface ListBlockData extends BaseBlockData {
  type: 'list';
  ordered: boolean;
  items: ListItemData[];
}

export interface TableCellData {
  children: InlineNodeData[];
}

export interface TableRowData {
  cells: TableCellData[];
}

export interface TableBlockData extends BaseBlockData {
  type: 'table';
  headers?: string[];
  rows: TableRowData[];
}

export interface ImageBlockData extends BaseBlockData {
  type: 'image';
  url: string;
  alt?: string;
  caption?: string;
  linkUrl?: string;
  width?: number;
  height?: number;
}

export interface CodeBlockData extends BaseBlockData {
  type: 'code';
  code: string;
  language?: string;
}

export interface QuoteBlockData extends BaseBlockData {
  type: 'quote';
  children: ContentBlockData[];
}

export interface CustomBlockData extends BaseBlockData {
  type: 'custom';
  content?: string;
  children?: ContentBlockData[];
}

export type ContentBlockData =
  | HeadingBlockData
  | ParagraphBlockData
  | ListBlockData
  | TableBlockData
  | ImageBlockData
  | CodeBlockData
  | QuoteBlockData
  | CustomBlockData
  | (BaseBlockData & Record<string, any>);

export interface ParsedArticleData {
  title?: string;
  blocks: ContentBlockData[];
  metadata?: Record<string, any>;
}
