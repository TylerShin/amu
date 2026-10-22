import AppKit

enum KeyTranslator {
    static func translate(_ event: NSEvent) -> CoreOperation {
        let shortcutFlags: NSEvent.ModifierFlags = [.command, .control, .option]
        if !event.modifierFlags.intersection(shortcutFlags).isEmpty {
            return .finish
        }
        if event.keyCode == 51 { return .backspace }
        if event.keyCode == 49 { return .wordBoundary }
        guard let letter = letters[event.keyCode], let ascii = letter.asciiValue else { return .finish }
        return .letter(event.modifierFlags.contains(.shift) ? ascii - 32 : ascii)
    }

    private static let letters: [UInt16: Character] = [
        0: "a", 1: "s", 2: "d", 3: "f", 4: "h", 5: "g",
        6: "z", 7: "x", 8: "c", 9: "v", 11: "b",
        12: "q", 13: "w", 14: "e", 15: "r", 16: "y", 17: "t",
        31: "o", 32: "u", 34: "i", 35: "p", 37: "l", 38: "j",
        40: "k", 45: "n", 46: "m",
    ]
}
