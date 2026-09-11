import type { ParsedArticle } from './FastHtmlParser.nitro';

export function estimateHtmlHeight(
  _html: string,
  _lineHeight: number = 22
): number {
  return 0;
}

export function parseHTML(_html: string): ParsedArticle | null {
  throw new Error(
    'react-native-fast-html-parser is not supported on this platform.'
  );
}

export function parseHTMLToJSON(_html: string): string {
  throw new Error(
    'react-native-fast-html-parser is not supported on this platform.'
  );
}

export async function parseHTMLAsync(
  _html: string
): Promise<ParsedArticle | null> {
  throw new Error(
    'react-native-fast-html-parser is not supported on this platform.'
  );
}
