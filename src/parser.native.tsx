import { NitroModules } from 'react-native-nitro-modules';
import type { FastHtmlParser, ParsedArticle } from './FastHtmlParser.nitro';

const FastHtmlParserHybridObject =
  NitroModules.createHybridObject<FastHtmlParser>('FastHtmlParser');

export function parseHTML(html: string): ParsedArticle | null {
  return FastHtmlParserHybridObject.parse(html);
}
