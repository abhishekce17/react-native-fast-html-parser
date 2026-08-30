import type { ParsedArticle } from './FastHtmlParser.nitro';

export function parseHTML(_html: string): ParsedArticle | null {
  throw new Error(
    "'react-native-fast-html-parser' is only supported on native platforms."
  );
}
