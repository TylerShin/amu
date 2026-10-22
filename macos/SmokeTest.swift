import AppKit

@main
@MainActor
struct SmokeTest {
    static func main() {
        _ = NSApplication.shared
        let input = TypingView(frame: NSRect(x: 0, y: 0, width: 600, height: 300))
        input.isRichText = false
        input.coordinator.changeMode(.korean, client: input)
        press(input, [(15, "r"), (40, "k"), (12, "q"), (17, "t"), (40, "k")])
        expect(input.string == "갑사", "Native marked text should show 갑사")
        press(input, [(51, "\u{7f}")])
        expect(input.string == "갑ㅅ", "Native Backspace must preserve the completed syllable")
        input.coordinator.finish(client: input)
        input.coordinator.finish(client: input)
        expect(input.string == "갑ㅅ" && !input.hasMarkedText(), "Repeated commit must not duplicate text")

        reset(input)
        input.coordinator.changeMode(.auto, client: input)
        press(input, [(4, "h"), (14, "e"), (37, "l"), (37, "l"), (31, "o")])
        input.coordinator.finish(client: input)
        input.insertText(" ", replacementRange: NSRange(location: NSNotFound, length: 0))
        press(input, [(2, "d"), (40, "k"), (1, "s"), (1, "s"), (32, "u"), (2, "d")])
        expect(input.string == "hello 안녕", "The Swift → C ABI → Rust → libhangul path must preserve mixed words")
        input.selectAll(nil)
        press(input, [(4, "h"), (34, "i")])
        input.coordinator.finish(client: input)
        expect(input.string == "hi", "Selection replacement must not retain the old composition")

        verifyMixedSuffix(input)
        verifyUnregisteredSuffixAndContext(input)
        verifyShiftAndRepeatedJamo(input)
        verifySpaceKeyPreservesWordBoundary(input)
        verifyShortReadingsAcrossContext(input)

        let coordinator = InputCoordinator()
        let reentrant = ReentrantClient(coordinator: coordinator)
        expect(coordinator.handle(keyEvent(15, "r"), client: reentrant), "Letter should be consumed")
        expect(reentrant.commits == ["r"], "Reentrant finish must commit exactly once")
        print("Native smoke passed: marked text, Backspace, repeated commit, 삼성sdi 사고싶어, unregistered qzx, word context, Shift overlap, repeated jamo, Space key, short readings across context, selection replacement, reentrant finish")
    }

    private static func verifyShortReadingsAcrossContext(_ input: TypingView) {
        let korean: [(UInt16, String)] = [(5, "g"), (40, "k"), (1, "s"), (15, "r"), (46, "m"), (3, "f"), (49, " ")]
        let english: [(UInt16, String)] = [(4, "h"), (14, "e"), (37, "l"), (37, "l"), (31, "o"), (49, " ")]
        let contexts = [(0, korean, ""), (1, korean, "한글 "), (4, korean, "한글 "), (1, english, "hello "), (4, english, "hello ")]
        let readings: [([(UInt16, String)], String)] = [
            ([(0, "a"), (34, "i")], "ai"),
            ([(0, "A"), (34, "I")], "AI"),
            ([(0, "A"), (34, "i")], "Ai"),
            ([(0, "a"), (34, "I")], "aI"),
            ([(9, "v"), (34, "i")], "vi"),
            ([(12, "q"), (34, "i")], "qi"),
            ([(15, "r"), (40, "k")], "가"),
            ([(2, "d"), (31, "O")], "얘"),
            ([(9, "v"), (38, "j"), (0, "a")], "펌"),
            ([(6, "z"), (45, "n"), (15, "r")], "쿡"),
        ]
        for (count, context, contextText) in contexts {
            for (keys, reading) in readings {
                reset(input)
                for _ in 0..<count { press(input, context) }
                let expected = String(repeating: contextText, count: count) + reading
                press(input, keys)
                expect(input.string == expected && input.hasMarkedText(), "Short reading before Space: \(expected), actual: \(input.string)")
                press(input, [(49, " ")])
                expect(input.string == expected + " " && !input.hasMarkedText(), "Short reading must commit once: \(expected)")
            }
        }
    }

    private static func verifySpaceKeyPreservesWordBoundary(_ input: TypingView) {
        reset(input)
        press(input, [(2, "d"), (38, "j"), (14, "E"), (38, "j"), (3, "f"), (15, "R"), (40, "k")])
        expect(input.string == "어떨까", "The first word must compose before Space")
        press(input, [(49, " ")])
        expect(input.string == "어떨까 " && !input.hasMarkedText(), "Space must commit the word and insert one space: \(input.string)")
        press(input, [(17, "t"), (37, "l"), (9, "v"), (1, "s"), (35, "p"), (2, "d"), (16, "y")])
        input.coordinator.finish(client: input)
        expect(input.string == "어떨까 싶네요", "Typing the next word must preserve the space: \(input.string)")
    }

    private static func verifyShiftAndRepeatedJamo(_ input: TypingView) {
        let cases: [([(UInt16, String)], String)] = [
            ([(12, "q"), (38, "j"), (3, "f"), (17, "T"), (38, "J")], "벌써"),
            ([(2, "d"), (40, "k"), (15, "R"), (40, "K")], "아까"),
            ([(2, "d"), (38, "j"), (14, "E"), (38, "j"), (3, "f"), (15, "R"), (40, "K")], "어떨까"),
            ([(2, "d"), (38, "j"), (14, "E"), (38, "J"), (3, "F"), (15, "R"), (40, "K")], "어떨까"),
            ([(5, "g"), (5, "g"), (5, "g")], "ㅎㅎㅎ"),
            ([(6, "z"), (6, "z"), (6, "z")], "ㅋㅋㅋ"),
        ]
        for (keys, expected) in cases {
            reset(input)
            press(input, keys)
            expect(input.string == expected && input.hasMarkedText(), "Shift and repeated jamo must survive the native event bridge: \(expected)")
            input.coordinator.finish(client: input)
            expect(input.string == expected && !input.hasMarkedText(), "Commit must preserve \(expected)")
        }
        press(input, [(5, "g"), (5, "g")])
        expect(input.string == "ㅋㅋㅋㅎㅎ", "Repeated jamo should compose after the previous token was committed")
        press(input, [(51, "\u{7f}")])
        expect(input.string == "ㅋㅋㅋㅎ", "Deleting repeated jamo must preserve the committed prefix")
    }

    private static func verifyUnregisteredSuffixAndContext(_ input: TypingView) {
        reset(input)
        press(input, [(17, "t"), (40, "k"), (0, "a"), (17, "t"), (38, "j"), (2, "d"), (12, "q"), (6, "z"), (7, "x")])
        expect(input.string == "삼성qzx", "An unregistered initialism must be recognized through the native bridge")
        reset(input)
        press(input, [(5, "g"), (40, "k"), (1, "s"), (15, "r"), (46, "m"), (3, "f")])
        expect(input.string == "한글", "Context setup should compose Korean")
        expect(!input.coordinator.handle(keyEvent(49, " "), client: input), "Word boundary should pass through")
        input.insertText(" ", replacementRange: NSRange(location: NSNotFound, length: 0))
        press(input, [(5, "g"), (31, "o")])
        expect(input.string == "한글 해", "The preceding Korean word must disambiguate go")
        reset(input)
        press(input, [(5, "g"), (31, "o")])
        expect(input.string == "go", "Reset must remove the previous client's language context")
    }

    private static func verifyMixedSuffix(_ input: TypingView) {
        reset(input)
        press(input, [(17, "t"), (40, "k"), (0, "a"), (17, "t"), (38, "j"), (2, "d"), (1, "s"), (2, "d"), (34, "i")])
        expect(input.string == "삼성sdi" && input.hasMarkedText(), "The mixed reading should replace the entire marked token")
        press(input, [(51, "\u{7f}")])
        expect(input.string == "삼성sd", "Backspace should erase only the English suffix")
        press(input, [(34, "i")])
        expect(!input.coordinator.handle(keyEvent(49, " "), client: input), "Space must commit the mixed token and pass through")
        input.insertText(" ", replacementRange: NSRange(location: NSNotFound, length: 0))
        press(input, [(17, "t"), (40, "k"), (15, "r"), (4, "h"), (17, "t"), (37, "l"), (9, "v"), (2, "d"), (38, "j")])
        input.coordinator.finish(client: input)
        input.coordinator.finish(client: input)
        expect(input.string == "삼성sdi 사고싶어" && !input.hasMarkedText(), "The complete sentence must commit once without losing either language")
    }

    private static func press(_ input: TypingView, _ keys: [(UInt16, String)]) {
        for (code, text) in keys { input.keyDown(with: keyEvent(code, text)) }
    }

    private static func keyEvent(_ code: UInt16, _ text: String) -> NSEvent {
        let flags: NSEvent.ModifierFlags = text != text.lowercased() ? [.shift] : []
        return NSEvent.keyEvent(with: .keyDown, location: .zero, modifierFlags: flags, timestamp: 0, windowNumber: 0, context: nil, characters: text, charactersIgnoringModifiers: text, isARepeat: false, keyCode: code)!
    }

    private static func reset(_ input: TypingView) {
        input.coordinator.finish(client: input)
        input.string = ""
        input.setSelectedRange(NSRange(location: 0, length: 0))
        input.coordinator.reset()
    }

    private static func expect(_ condition: Bool, _ message: String) {
        guard condition else {
            FileHandle.standardError.write(Data((message + "\n").utf8))
            exit(1)
        }
    }
}

@MainActor
private final class ReentrantClient: CompositionClient {
    let coordinator: InputCoordinator
    var commits: [String] = []

    init(coordinator: InputCoordinator) { self.coordinator = coordinator }
    func mark(_ text: String) { coordinator.finish(client: self) }
    func commit(_ text: String) {
        commits.append(text)
        coordinator.finish(client: self)
    }
}
