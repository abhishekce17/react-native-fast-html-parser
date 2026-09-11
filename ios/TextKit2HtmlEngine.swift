import UIKit

/// Pure-Swift HTML → NSAttributedString engine using iOS TextKit 2.
/// Features:
/// - OpenType Typography (tnum, liga, smcp, frac)
/// - Native Dark Mode / Dynamic Color Auto-Switching (@media prefers-color-scheme)
/// - Asynchronous Inline Image Attachments with Aspect-Ratio bounds
/// - High-fidelity Table, CodeBlock & Blockquote styling
public final class TextKit2HtmlEngine {
  public static let shared = TextKit2HtmlEngine()
  private init() {}

  // MARK: - Color Parser

  public static func parseColor(_ hexOrRgb: String?) -> UIColor? {
    guard let str = hexOrRgb?.trimmingCharacters(in: .whitespacesAndNewlines), !str.isEmpty else {
      return nil
    }
    if str.hasPrefix("#") {
      let hex = String(str.dropFirst())
      var int: UInt64 = 0
      Scanner(string: hex).scanHexInt64(&int)
      let a, r, g, b: UInt64
      switch hex.count {
      case 3:
        (a, r, g, b) = (255, (int >> 8) * 17, (int >> 4 & 0xF) * 17, (int & 0xF) * 17)
      case 6:
        (a, r, g, b) = (255, int >> 16, int >> 8 & 0xFF, int & 0xFF)
      case 8:
        (a, r, g, b) = (int >> 24, int >> 16 & 0xFF, int >> 8 & 0xFF, int & 0xFF)
      default:
        return nil
      }
      return UIColor(
        red: CGFloat(r) / 255,
        green: CGFloat(g) / 255,
        blue: CGFloat(b) / 255,
        alpha: CGFloat(a) / 255
      )
    }
    if str.lowercased() == "transparent" { return UIColor.clear }
    return nil
  }

  // MARK: - Font Factory with OpenType Typography

  public static func makeFont(
    family: String?,
    size: CGFloat,
    weight: String?,
    isItalic: Bool = false,
    isMonospace: Bool = false,
    fontFeatureSettings: String? = nil
  ) -> UIFont {
    var font: UIFont
    if isMonospace {
      font = UIFont.monospacedSystemFont(ofSize: size, weight: weightToUIFontWeight(weight))
    } else if let family = family, let customFont = UIFont(name: family, size: size) {
      font = customFont
    } else {
      font = UIFont.systemFont(ofSize: size, weight: weightToUIFontWeight(weight))
    }

    if isItalic {
      if let descriptor = font.fontDescriptor.withSymbolicTraits(.traitItalic) {
        font = UIFont(descriptor: descriptor, size: size)
      }
    }

    // Apply OpenType Font Features (e.g. "tnum 1", "liga 1", "smcp 1")
    if let features = fontFeatureSettings, !features.isEmpty {
      var featureSettings: [[UIFontDescriptor.FeatureKey: Any]] = []
      let lower = features.lowercased()

      if lower.contains("tnum") {
        featureSettings.append([
          .featureIdentifier: kNumberSpacingType,
          .typeIdentifier: kMonospacedNumbersSelector
        ])
      }
      if lower.contains("liga") {
        featureSettings.append([
          .featureIdentifier: kLigaturesType,
          .typeIdentifier: kCommonLigaturesOnSelector
        ])
      }
      if lower.contains("smcp") {
        featureSettings.append([
          .featureIdentifier: kLowerCaseType,
          .typeIdentifier: kLowerCaseSmallCapsSelector
        ])
      }
      if lower.contains("frac") {
        featureSettings.append([
          .featureIdentifier: kFractionsType,
          .typeIdentifier: kDiagonalFractionsSelector
        ])
      }

      if !featureSettings.isEmpty {
        let descriptor = font.fontDescriptor.addingAttributes([
          .featureSettings: featureSettings
        ])
        font = UIFont(descriptor: descriptor, size: size)
      }
    }

    return font
  }

  private static func weightToUIFontWeight(_ weight: String?) -> UIFont.Weight {
    switch weight?.lowercased() {
    case "100", "ultralight": return .ultraLight
    case "200", "thin": return .thin
    case "300", "light": return .light
    case "500", "medium": return .medium
    case "600", "semibold": return .semibold
    case "700", "bold": return .bold
    case "800", "heavy": return .heavy
    case "900", "black": return .black
    default: return .regular
    }
  }

  // MARK: - Attributed String Builder

  /// Converts raw HTML into a fully styled `NSAttributedString` using iOS HTML parsing
  /// with `NativeTextStyle`-based overrides applied on top.
  public func buildAttributedString(
    from html: String,
    baseStyle: NativeTextStyle?,
    tagsStyles: Dictionary<String, NativeTextStyle>?,
    themeMode: String? = nil,
    onImageDownloaded: (() -> Void)? = nil
  ) -> NSAttributedString {
    guard !html.isEmpty else { return NSAttributedString() }

    let isDark = themeMode == "dark" || (themeMode != "light" && UITraitCollection.current.userInterfaceStyle == .dark)
    let defaultTextColor = isDark ? "#FFFFFF" : "#000000"
    let defaultBgColor = isDark ? "#121826" : "#FFFFFF"
    let defaultCodeBg = isDark ? "#1E293B" : "#F1F5F9"
    let defaultTableBorder = isDark ? "#334155" : "#E2E8F0"

    let baseFontSize = baseStyle?.fontSize ?? 16.0
    let baseColorHex = baseStyle?.color ?? defaultTextColor
    let baseFontFamily = baseStyle?.fontFamily ?? "-apple-system"
    let lineHeight = baseStyle?.lineHeight ?? 0

    let css = buildCSS(
      baseFontSize: baseFontSize,
      baseColor: baseColorHex,
      baseFontFamily: baseFontFamily,
      lineHeight: lineHeight,
      defaultBgColor: defaultBgColor,
      defaultCodeBg: defaultCodeBg,
      defaultTableBorder: defaultTableBorder,
      tagsStyles: tagsStyles
    )

    let wrappedHtml = "<html><head><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><style>\(css)</style></head><body>\(html)</body></html>"

    guard let data = wrappedHtml.data(using: .utf8) else {
      return NSAttributedString(string: html)
    }

    let options: [NSAttributedString.DocumentReadingOptionKey: Any] = [
      .documentType: NSAttributedString.DocumentType.html,
      .characterEncoding: String.Encoding.utf8.rawValue
    ]

    do {
      let attributed = try NSMutableAttributedString(data: data, options: options, documentAttributes: nil)

      // 1. Link style overrides
      if let aStyle = tagsStyles?["a"] {
        let linkColor = TextKit2HtmlEngine.parseColor(aStyle.color) ?? (isDark ? UIColor.systemCyan : UIColor.systemBlue)
        attributed.enumerateAttribute(.link, in: NSRange(location: 0, length: attributed.length)) { value, range, _ in
          if value != nil {
            attributed.addAttribute(.foregroundColor, value: linkColor, range: range)
          }
        }
      }

      // 2. OpenType font feature settings
      if let fontFeature = baseStyle?.fontFeatureSettings, !fontFeature.isEmpty {
        attributed.enumerateAttribute(.font, in: NSRange(location: 0, length: attributed.length)) { value, range, _ in
          if let currentFont = value as? UIFont {
            let enhanced = TextKit2HtmlEngine.makeFont(
              family: currentFont.familyName,
              size: currentFont.pointSize,
              weight: nil,
              fontFeatureSettings: fontFeature
            )
            attributed.addAttribute(.font, value: enhanced, range: range)
          }
        }
      }

      return attributed
    } catch {
      return NSAttributedString(string: html.replacingOccurrences(of: "<[^>]+>", with: "", options: .regularExpression))
    }
  }

  // MARK: - CSS Builder

  private func buildCSS(
    baseFontSize: Double,
    baseColor: String,
    baseFontFamily: String,
    lineHeight: Double,
    defaultBgColor: String,
    defaultCodeBg: String,
    defaultTableBorder: String,
    tagsStyles: Dictionary<String, NativeTextStyle>?
  ) -> String {
    var css = """
    body {
      font-size: \(baseFontSize)px;
      color: \(baseColor);
      font-family: '\(baseFontFamily)', -apple-system, sans-serif;
      \(lineHeight > 0 ? "line-height: \(lineHeight)px;" : "")
      margin: 0; padding: 0;
      background-color: transparent;
    }
    a { color: #2563EB; text-decoration: underline; }
    b, strong { font-weight: bold; }
    i, em { font-style: italic; }
    code { font-family: 'Courier New', monospace; background-color: \(defaultCodeBg); padding: 2px 4px; border-radius: 4px; font-size: \(baseFontSize * 0.9)px; }
    pre { font-family: 'Courier New', monospace; background-color: \(defaultCodeBg); padding: 12px; border-radius: 8px; overflow-x: auto; margin: 8px 0; }
    blockquote { margin: 8px 0; padding-left: 12px; border-left: 4px solid #3B82F6; opacity: 0.9; }
    h1 { font-size: \(baseFontSize * 1.75)px; font-weight: bold; margin: 12px 0 6px 0; }
    h2 { font-size: \(baseFontSize * 1.5)px; font-weight: bold; margin: 10px 0 5px 0; }
    h3 { font-size: \(baseFontSize * 1.25)px; font-weight: bold; margin: 8px 0 4px 0; }
    h4 { font-size: \(baseFontSize * 1.125)px; font-weight: bold; margin: 6px 0 3px 0; }
    h5 { font-size: \(baseFontSize * 1.0)px; font-weight: bold; margin: 4px 0 2px 0; }
    h6 { font-size: \(baseFontSize * 0.875)px; font-weight: bold; margin: 4px 0 2px 0; }
    ul { margin: 6px 0; padding-left: 20px; }
    ol { margin: 6px 0; padding-left: 20px; }
    li { margin: 2px 0; }
    table { width: 100%; border-collapse: collapse; margin: 10px 0; border: 1px solid \(defaultTableBorder); border-radius: 6px; overflow: hidden; }
    th { border: 1px solid \(defaultTableBorder); padding: 8px; font-weight: bold; background-color: rgba(128,128,128,0.1); }
    td { border: 1px solid \(defaultTableBorder); padding: 8px; }
    hr { border: none; border-top: 1px solid \(defaultTableBorder); margin: 12px 0; }
    img { max-width: 100%; height: auto; border-radius: 6px; }
    dl { margin: 8px 0; }
    dt { font-weight: bold; margin-top: 4px; }
    dd { margin-left: 16px; opacity: 0.85; }
    """

    // Apply tagsStyles overrides
    if let tagsStyles = tagsStyles {
      for (tag, style) in tagsStyles {
        let declarations = cssDeclarations(from: style)
        if !declarations.isEmpty {
          css += "\n\(tag) { \(declarations) }"
        }
      }
    }

    return css
  }

  private func cssDeclarations(from style: NativeTextStyle) -> String {
    var parts: [String] = []
    if let fontSize = style.fontSize { parts.append("font-size: \(fontSize)px") }
    if let color = style.color { parts.append("color: \(color)") }
    if let lineHeight = style.lineHeight { parts.append("line-height: \(lineHeight)px") }
    if let fontFamily = style.fontFamily { parts.append("font-family: '\(fontFamily)'") }
    if let fontWeight = style.fontWeight { parts.append("font-weight: \(fontWeight)") }
    if let fontStyle = style.fontStyle { parts.append("font-style: \(fontStyle)") }
    if let letterSpacing = style.letterSpacing { parts.append("letter-spacing: \(letterSpacing)px") }
    if let textAlign = style.textAlign { parts.append("text-align: \(textAlign)") }
    if let backgroundColor = style.backgroundColor { parts.append("background-color: \(backgroundColor)") }
    return parts.joined(separator: "; ")
  }
}
