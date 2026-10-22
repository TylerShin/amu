import AppKit

/// The host owns the font and colors; Amu owns only the composition underline.
struct MarkedText {
    let attributedString: NSAttributedString
    let selectionRange: NSRange

    init(_ text: String) {
        attributedString = NSAttributedString(string: text, attributes: [
            .underlineStyle: NSUnderlineStyle.single.rawValue,
            .markedClauseSegment: 0,
        ])
        selectionRange = NSRange(location: attributedString.length, length: 0)
    }
}
