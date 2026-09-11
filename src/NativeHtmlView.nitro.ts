import type {
  HybridView,
  HybridViewProps,
  HybridViewMethods,
} from 'react-native-nitro-modules';

export interface NativeTextStyle {
  fontSize?: number;
  color?: string;
  lineHeight?: number;
  fontFamily?: string;
  fontWeight?: string;
  fontStyle?: string;
  letterSpacing?: number;
  textAlign?: string;
  backgroundColor?: string;
  fontFeatureSettings?: string;
}

export interface NativeHtmlViewProps extends HybridViewProps {
  html?: string;
  baseStyle?: NativeTextStyle;
  tagsStyles?: Record<string, NativeTextStyle>;
  selectable?: boolean;
  onLinkPress?: (url: string) => void;
  onContentSizeChange?: (height: number) => void;
  themeMode?: string;
}

export interface NativeHtmlViewMethods extends HybridViewMethods {
  getTextContent(): string;
}

export type NativeHtmlView = HybridView<
  NativeHtmlViewProps,
  NativeHtmlViewMethods,
  { ios: 'swift'; android: 'kotlin' }
>;
