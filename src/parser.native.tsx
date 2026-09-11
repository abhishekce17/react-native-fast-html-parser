import { NitroModules } from 'react-native-nitro-modules';
import type { FastHtmlParser, ParsedArticle } from './FastHtmlParser.nitro';

export const FastHtmlParserInstance =
  NitroModules.createHybridObject<FastHtmlParser>('FastHtmlParser');

export function estimateHtmlHeight(
  html: string,
  lineHeight: number = 22
): number {
  return FastHtmlParserInstance.estimateHeight(html, lineHeight);
}

export function parseHTML(html: string): ParsedArticle | null {
  return FastHtmlParserInstance.parse(html);
}

export function parseHTMLToJSON(html: string): string {
  return FastHtmlParserInstance.parseToJSON(html);
}

export async function parseHTMLAsync(
  html: string
): Promise<ParsedArticle | null> {
  return FastHtmlParserInstance.parseAsync(html);
}
