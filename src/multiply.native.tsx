import { NitroModules } from 'react-native-nitro-modules';
import type { FastHtmlParser } from './FastHtmlParser.nitro';

const FastHtmlParserHybridObject =
  NitroModules.createHybridObject<FastHtmlParser>('FastHtmlParser');

export function multiply(a: number, b: number): number {
  return FastHtmlParserHybridObject.multiply(a, b);
}
