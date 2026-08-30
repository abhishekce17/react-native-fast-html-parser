import type { HybridObject } from 'react-native-nitro-modules';

export interface InlineNode extends HybridObject<{ ios: 'c++', android: 'c++' }> {
  readonly type: string;
  readonly text: string;
  readonly url: string;
  readonly childCount: number;
  getChild(index: number): InlineNode | null;
}

export interface ListItem extends HybridObject<{ ios: 'c++', android: 'c++' }> {
  readonly childCount: number;
  getChild(index: number): InlineNode | null;
  readonly nestedCount: number;
  getNested(index: number): ContentBlock | null;
}

export interface TableCell extends HybridObject<{ ios: 'c++', android: 'c++' }> {
  readonly childCount: number;
  getChild(index: number): InlineNode | null;
}

export interface TableRow extends HybridObject<{ ios: 'c++', android: 'c++' }> {
  readonly cellCount: number;
  getCell(index: number): TableCell | null;
}

export interface DefinitionItem extends HybridObject<{ ios: 'c++', android: 'c++' }> {
  readonly termCount: number;
  getTerm(index: number): InlineNode | null;
  readonly defCount: number;
  getDef(index: number): InlineNode | null;
}

export interface ContentBlock extends HybridObject<{ ios: 'c++', android: 'c++' }> {
  readonly type: string;
  readonly level: number;
  readonly url: string;
  readonly alt: string;
  readonly caption: string;
  readonly code: string;
  readonly language: string;
  readonly src: string;
  readonly poster: string;
  readonly title: string;

  readonly childCount: number;
  getChild(index: number): InlineNode | null;
  getQuoteChild(index: number): ContentBlock | null;

  readonly ordered: boolean;
  readonly itemCount: number;
  getItem(index: number): ListItem | null;

  readonly rowCount: number;
  getRow(index: number): TableRow | null;

  getDefItem(index: number): DefinitionItem | null;
}

export interface ParsedArticle extends HybridObject<{ ios: 'c++', android: 'c++' }> {
  readonly length: number;
  getBlock(index: number): ContentBlock | null;
}

export interface FastHtmlParser extends HybridObject<{ ios: 'c++', android: 'c++' }> {
  parse(html: string): ParsedArticle | null;
}
