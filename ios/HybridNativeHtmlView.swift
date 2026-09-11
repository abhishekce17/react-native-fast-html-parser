import UIKit
import NitroModules

/// Private NSObject shim that acts as UITextViewDelegate.
private final class TextViewDelegateShim: NSObject, UITextViewDelegate {
  weak var owner: HybridNativeHtmlView?

  func textView(
    _ textView: UITextView,
    shouldInteractWith URL: URL,
    in characterRange: NSRange,
    interaction: UITextItemInteraction
  ) -> Bool {
    if let onLinkPress = owner?.onLinkPress {
      onLinkPress(URL.absoluteString)
      return false
    }
    return true
  }

  @available(iOS 17.0, *)
  func textView(
    _ textView: UITextView,
    primaryActionFor textItem: UITextItem,
    defaultAction: UIAction
  ) -> UIAction? {
    if case .link(let url) = textItem.content, let onLinkPress = owner?.onLinkPress {
      return UIAction { _ in onLinkPress(url.absoluteString) }
    }
    return defaultAction
  }
}

/// Native HTML view backed by iOS TextKit 2, implementing the Nitrogen-generated HybridView spec.
open class HybridNativeHtmlView: HybridNativeHtmlViewSpec_base, HybridNativeHtmlViewSpec_protocol {

  // MARK: - HybridView

  public let textView: UITextView
  private let delegateShim = TextViewDelegateShim()

  public var view: UITextView {
    return textView
  }

  // MARK: - Props

  public var html: String? {
    didSet { setNeedsContentUpdate() }
  }

  public var baseStyle: NativeTextStyle? {
    didSet { setNeedsContentUpdate() }
  }

  public var tagsStyles: Dictionary<String, NativeTextStyle>? {
    didSet { setNeedsContentUpdate() }
  }

  public var selectable: Bool? {
    didSet { textView.isSelectable = selectable ?? true }
  }

  public var themeMode: String? {
    didSet { setNeedsContentUpdate() }
  }

  public var onLinkPress: ((_ url: String) -> Void)?
  public var onContentSizeChange: ((_ height: Double) -> Void)?

  // MARK: - Init

  public override init() {
    self.textView = UITextView()
    super.init()
    delegateShim.owner = self
    setupTextView()
  }

  private func setupTextView() {
    textView.isEditable = false
    textView.isScrollEnabled = false
    textView.isSelectable = true
    textView.backgroundColor = .clear
    textView.textContainer.lineFragmentPadding = 0
    textView.textContainerInset = .zero
    textView.delegate = delegateShim
    textView.dataDetectorTypes = []
    textView.linkTextAttributes = [
      .foregroundColor: UIColor.systemBlue,
      .underlineStyle: NSUnderlineStyle.single.rawValue as Any
    ]
  }

  // MARK: - Background Pre-Layout & Content Update

  private var isUpdateScheduled = false

  private func setNeedsContentUpdate() {
    guard !isUpdateScheduled else { return }
    isUpdateScheduled = true

    DispatchQueue.main.async { [weak self] in
      guard let self = self else { return }
      self.isUpdateScheduled = false
      self.updateContent()
    }
  }

  private func updateContent() {
    guard let rawHtml = html, !rawHtml.isEmpty else {
      textView.attributedText = NSAttributedString(string: "")
      onContentSizeChange?(0)
      return
    }

    let base = baseStyle
    let tags = tagsStyles
    let mode = themeMode

    let attr = TextKit2HtmlEngine.shared.buildAttributedString(
      from: rawHtml,
      baseStyle: base,
      tagsStyles: tags,
      themeMode: mode,
      onImageDownloaded: { [weak self] in
        self?.textView.setNeedsDisplay()
      }
    )
    textView.attributedText = attr
    updateAccessibility(for: attr)

    let targetWidth = textView.bounds.width > 0 ? textView.bounds.width : (UIScreen.main.bounds.width - 64)
    let size = textView.sizeThatFits(CGSize(width: targetWidth, height: .greatestFiniteMagnitude))
    onContentSizeChange?(Double(max(16, ceil(size.height))))
  }

  // MARK: - Web-Standard Accessibility Tree (VoiceOver)

  private func updateAccessibility(for attributedString: NSAttributedString) {
    var elements: [UIAccessibilityElement] = []

    // 1. Whole view container
    let container = UIAccessibilityElement(accessibilityContainer: textView)
    container.accessibilityLabel = attributedString.string
    container.accessibilityFrameInContainerSpace = textView.bounds
    elements.append(container)

    // 2. Discover links for VoiceOver Links Rotor
    attributedString.enumerateAttribute(.link, in: NSRange(location: 0, length: attributedString.length)) { [weak self] value, range, _ in
      guard let self = self, let url = value as? URL else { return }
      let linkElement = UIAccessibilityElement(accessibilityContainer: self.textView)
      let substring = (attributedString.string as NSString).substring(with: range)
      linkElement.accessibilityLabel = substring
      linkElement.accessibilityValue = url.absoluteString
      linkElement.accessibilityTraits = [.link]
      elements.append(linkElement)
    }

    textView.accessibilityElements = elements
  }

  // MARK: - HybridObject Methods

  public func getTextContent() throws -> String {
    return textView.text ?? ""
  }
}
