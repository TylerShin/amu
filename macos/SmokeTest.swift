import AppKit
import Carbon
import IOKit.hidsystem

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
        verifyEnglishFirstBoundaries(input)
        verifyConnectedEnglishSpelling(input)
        verifyPerKeyPresentation(input)
        verifyCompetingShortReadings(input)
        verifyMarkedTextPresentation()
        verifySecureInput()
        verifyManualReadings(input)
        verifyRightShiftCorrection(input)
        verifyPortableShiftFlags(input)
        verifyCorrectionGestureBoundaries()
        verifyModeShortcuts()

        let coordinator = InputCoordinator()
        let reentrant = ReentrantClient(coordinator: coordinator)
        expect(coordinator.handle(keyEvent(15, "r"), client: reentrant), "Letter should be consumed")
        expect(reentrant.commits == ["r"], "Reentrant finish must commit exactly once")
        print("Native smoke passed: marked text, Backspace, repeated commit, 삼성sdi 사고싶어, unregistered qzx, word context, Shift overlap, repeated jamo, Space key, short readings across context, English-first boundaries, connected English spelling, initial expressions, per-key presentation, explicit composition style, secure input, selection replacement, reentrant finish, manual reading correction, right Shift tap, mode shortcuts")
    }

    private static func verifyPerKeyPresentation(_ input: TypingView) {
        let cases: [[(UInt16, String, String)]] = [
            [(37, "l", "l"), (31, "o", "lo"), (8, "c", "loc"), (40, "k", "lock")],
            [(5, "g", "g"), (40, "k", "하"), (1, "s", "한"), (15, "r", "한ㄱ"), (46, "m", "한그"), (3, "f", "한글")],
            [(15, "r", "r"), (17, "t", "ㄱㅅ"), (15, "r", "ㄱㅅㄱ"), (17, "t", "ㄱㅅㄱㅅ")]
        ]
        for keys in cases {
            reset(input)
            input.coordinator.changeMode(.auto, client: input)
            for (code, letter, expected) in keys {
                press(input, [(code, letter)])
                expect(input.string == expected, "Each native key must show \(expected), got \(input.string)")
                expect(input.hasMarkedText(), "Partial input must remain marked")
                expect(input.selectedRange().location == expected.utf16.count, "Caret must follow the displayed text")
            }
            let shown = input.string
            press(input, [(49, " ")])
            expect(input.string == shown + " " && !input.hasMarkedText(), "Space must commit the visible frame once")
        }
    }

    private static func verifyRightShiftCorrection(_ input: TypingView) {
        reset(input)
        input.coordinator.changeMode(.auto, client: input)
        press(input, [(5, "g"), (31, "o"), (0, "a"), (37, "l")])
        expect(input.string == "goal", "A complete English word must beat unrelated familiar syllables")
        input.flagsChanged(with: shiftEvent(pressed: true))
        expect(input.string == "goal", "Pressing Shift alone must not change the word before release")
        input.flagsChanged(with: shiftEvent(pressed: false))
        expect(input.string == "해미" && input.hasMarkedText(), "One tap must select the whole Korean alternative")
        input.flagsChanged(with: shiftEvent(pressed: true))
        input.flagsChanged(with: shiftEvent(pressed: false))
        expect(input.string == "goal" && input.selectedRange().location == 4, "A second tap must return to English in place")
        press(input, [(49, " "), (2, "d"), (40, "k"), (1, "s"), (1, "s"), (32, "u"), (2, "d")])
        expect(input.string == "goal 안녕", "The next word must return to automatic selection")

        reset(input)
        press(input, [(2, "d"), (38, "j")])
        input.flagsChanged(with: shiftEvent(pressed: true))
        input.keyDown(with: keyEvent(17, "T", flags: [.shift, NSEvent.ModifierFlags(rawValue: UInt(NX_DEVICERSHIFTKEYMASK))]))
        input.flagsChanged(with: shiftEvent(pressed: false))
        expect(input.string == "었", "Right Shift used for a tense Korean consonant must not correct on release")

        reset(input)
        input.flagsChanged(with: shiftEvent(pressed: true))
        for (code, letter) in [(UInt16(0), "A"), (34, "I")] {
            input.keyDown(with: keyEvent(code, letter, flags: [.shift, NSEvent.ModifierFlags(rawValue: UInt(NX_DEVICERSHIFTKEYMASK))]))
        }
        input.flagsChanged(with: shiftEvent(pressed: false))
        expect(input.string == "AI", "Right Shift used for capitals must not convert AI on release")
    }

    private static func verifyPortableShiftFlags(_ input: TypingView) {
        reset(input)
        input.coordinator.changeMode(.auto, client: input)
        press(input, [(2, "d"), (34, "i"), (0, "a")])
        // IME transport does not promise the device-dependent low 16 bits.
        input.flagsChanged(with: shiftEvent(pressed: true, deviceBits: false))
        expect(input.string == "얌", "Portable Shift press must wait for release")
        input.flagsChanged(with: shiftEvent(pressed: false, deviceBits: false))
        expect(input.string == "dia" && input.hasMarkedText(), "Right Shift must correct with only standard modifier flags")

        let rightDown = shiftEvent(pressed: true, deviceBits: false)
        let rightUp = shiftEvent(pressed: false, deviceBits: false)
        let leftDown = shiftEvent(pressed: true, code: 56, deviceBits: false)
        let leftUp = shiftEvent(pressed: false, code: 56, deviceBits: false)
        for keys in [
            [leftDown, leftUp],
            [leftDown, rightDown, rightDown, leftUp],
            [rightDown, leftDown, leftDown, rightUp],
            [rightDown, keyEvent(0, "A", flags: [.shift]), rightDown, rightUp],
            [shiftEvent(pressed: true, additional: [.option], deviceBits: false), rightUp],
            [shiftEvent(pressed: true, code: 0, deviceBits: false), rightUp]
        ] {
            var gesture = CorrectionGesture()
            for key in keys {
                expect(!gesture.observe(key, canCorrect: true), "Left, overlapping, repeated, or unknown modifiers must not trigger correction")
            }
            expect(!gesture.observe(rightDown, canCorrect: true), "A new tap must wait for release")
            expect(!gesture.observe(rightDown, canCorrect: true), "A duplicate press must wait for release")
            expect(gesture.observe(rightUp, canCorrect: true), "A fresh solo tap after a chord must work")
            expect(!gesture.observe(rightUp, canCorrect: true), "A duplicate release must do nothing")
        }
    }

    private static func verifyCorrectionGestureBoundaries() {
        var secure = false
        let coordinator = InputCoordinator(secureInputEnabled: { secure })
        let client = PresentationClient()
        let down = shiftEvent(pressed: true)
        let up = shiftEvent(pressed: false)
        expect(!coordinator.handle(down, client: client) && !coordinator.handle(up, client: client), "An empty tap must pass through")

        for (code, letter) in [(UInt16(2), "d"), (34, "i"), (0, "a")] {
            _ = coordinator.handle(keyEvent(code, letter), client: client)
        }
        for other in [
            shiftEvent(pressed: true, additional: [.control], code: 59),
            shiftEvent(pressed: true, additional: [.command], code: 55),
            shiftEvent(pressed: true, additional: [NSEvent.ModifierFlags(rawValue: UInt(NX_DEVICELSHIFTKEYMASK))], code: 56)
        ] {
            _ = coordinator.handle(down, client: client)
            _ = coordinator.handle(other, client: client)
            expect(!coordinator.handle(up, client: client), "Another modifier must cancel the tap")
            expect(coordinator.markedText == "얌", "A modifier chord must preserve the word")
        }
        let both = shiftEvent(pressed: true, additional: [NSEvent.ModifierFlags(rawValue: UInt(NX_DEVICELSHIFTKEYMASK))])
        _ = coordinator.handle(both, client: client)
        expect(!coordinator.handle(up, client: client), "Right Shift pressed while left Shift is held is not a solo tap")

        _ = coordinator.handle(down, client: client)
        coordinator.finish(client: client)
        expect(!coordinator.handle(up, client: client), "Finishing must cancel an armed tap")
        expect(client.commits == ["얌"], "A late release must not rewrite committed text")

        _ = coordinator.handle(keyEvent(2, "d"), client: client)
        _ = coordinator.handle(down, client: client)
        let click = NSEvent.mouseEvent(with: .leftMouseDown, location: .zero, modifierFlags: [], timestamp: 0, windowNumber: 0, context: nil, eventNumber: 1, clickCount: 1, pressure: 1)!
        expect(!coordinator.handle(click, client: client), "Click must pass through after finishing the current word")
        expect(!coordinator.handle(up, client: client), "Click must cancel an armed Shift tap before the caret moves")
        expect(client.commits == ["얌", "d"], "Click must commit the visible text once")

        _ = coordinator.handle(keyEvent(2, "d"), client: client)
        _ = coordinator.handle(down, client: client)
        secure = true
        let password = PresentationClient()
        expect(!coordinator.handle(up, client: password), "Secure input must cancel and bypass the gesture")
        expect(password.lastMarked == nil && password.commits.isEmpty, "No gesture callback may write into a password field")
        secure = false
        coordinator.changeMode(.english, client: client)
        _ = coordinator.handle(keyEvent(2, "d"), client: client)
        _ = coordinator.handle(down, client: client)
        expect(!coordinator.handle(up, client: client) && coordinator.markedText == "d", "Fixed mode must ignore the automatic correction gesture")

        coordinator.changeMode(.auto, client: client)
        for (code, letter) in [(UInt16(2), "d"), (34, "i"), (0, "a")] {
            _ = coordinator.handle(keyEvent(code, letter), client: client)
        }
        client.commits.removeAll()
        client.onMark = { coordinator.finish(client: client) }
        _ = coordinator.handle(down, client: client)
        expect(coordinator.handle(up, client: client), "The correction release must be consumed")
        expect(client.commits == ["dia"], "Reentrant correction must commit exactly once")
        client.onMark = nil
        expect(!coordinator.handle(up, client: client), "A repeated release must not trigger a second correction")
    }

    private static func shiftEvent(pressed: Bool, additional: NSEvent.ModifierFlags = [], code: UInt16 = 60, deviceBits: Bool = true) -> NSEvent {
        var flags = additional
        if pressed {
            flags.insert(.shift)
            if deviceBits { flags.insert(NSEvent.ModifierFlags(rawValue: UInt(NX_DEVICERSHIFTKEYMASK))) }
        }
        return NSEvent.keyEvent(with: .flagsChanged, location: .zero, modifierFlags: flags, timestamp: 0, windowNumber: 0, context: nil, characters: "", charactersIgnoringModifiers: "", isARepeat: false, keyCode: code)!
    }

    private static func verifyManualReadings(_ input: TypingView) {
        reset(input)
        input.coordinator.changeMode(.auto, client: input)
        press(input, [(2, "d"), (34, "i"), (0, "a")])
        expect(input.string == "얌", "Reproduce an unfamiliar Latin name before manual correction")
        let correction = keyEvent(49, " ", flags: [.control, .shift])
        expect(input.coordinator.handle(correction, client: input), "The correction shortcut must not insert Space")
        expect(input.string == "dia" && input.hasMarkedText(), "Correct the current marked word in place")
        expect(input.coordinator.handle(keyEvent(49, " ", flags: [.control, .shift], repeatKey: true), client: input), "Holding the shortcut must be consumed")
        expect(input.string == "dia", "Key repeat must not cycle interpretations repeatedly")
        press(input, [(51, "\u{7f}"), (0, "a")])
        expect(input.string == "dia", "A manually chosen Latin reading must survive editing")
        press(input, [(49, " "), (2, "d"), (40, "k"), (1, "s"), (1, "s"), (32, "u"), (2, "d")])
        expect(input.string == "dia 안녕", "The following word must return to automatic selection")
        input.coordinator.finish(client: input)
        input.coordinator.finish(client: input)
        expect(input.string == "dia 안녕" && !input.hasMarkedText(), "Manual correction must not duplicate committed text")
        _ = input.coordinator.handle(correction, client: input)
        expect(input.string == "dia 안녕" && !input.hasMarkedText(), "The shortcut must not rewrite a committed word")

        reset(input)
        press(input, [(3, "f"), (2, "d")])
        expect(input.string == "ㄹㅇ", "Normal initials must keep their existing interpretation")
        _ = input.coordinator.handle(correction, client: input)
        expect(input.string == "fd", "The same initials can be explicitly chosen as Latin")
        _ = input.coordinator.handle(correction, client: input)
        expect(input.string == "ㄹㅇ", "The user can return to the Korean initials")
        press(input, [(51, "\u{7f}"), (40, "k")])
        expect(input.string == "라", "Corrected initials must still compose a syllable")

        reset(input)
        press(input, [(15, "r"), (40, "k"), (12, "q"), (17, "t"), (40, "k"), (51, "\u{7f}")])
        _ = input.coordinator.handle(correction, client: input)
        expect(input.string == "갑ㅅ", "Manual correction must preserve edited syllable boundaries")

        let coordinator = InputCoordinator()
        let client = PresentationClient()
        for (code, text) in [(UInt16(2), "d"), (34, "i"), (0, "a")] {
            _ = coordinator.handle(keyEvent(code, text), client: client)
        }
        client.onMark = { coordinator.finish(client: client) }
        _ = coordinator.handle(correction, client: client)
        expect(client.commits == ["dia"], "A reentrant host must commit the corrected reading exactly once")
        client.onMark = nil
    }

    private static func verifyModeShortcuts() {
        let coordinator = InputCoordinator()
        let client = PresentationClient()
        var selected: [InputMode] = []
        coordinator.modeDidChange = { selected.append($0) }
        let shortcuts: [(UInt16, InputMode, String)] = [(99, .english, "gksrmf"), (120, .korean, "한글"), (122, .auto, "한글")]
        let keys: [(UInt16, String)] = [(5, "g"), (40, "k"), (1, "s"), (15, "r"), (46, "m"), (3, "f")]
        for (code, mode, expected) in shortcuts {
            expect(coordinator.handle(keyEvent(code, "", flags: [.control, .shift, .function]), client: client), "Mode selection must consume its function key")
            expect(coordinator.mode == mode, "The shortcut must select the requested mode directly")
            for _ in 0..<2 {
                for (key, text) in keys { _ = coordinator.handle(keyEvent(key, text), client: client) }
                expect(coordinator.markedText == expected, "A fixed mode must survive word boundaries")
                _ = coordinator.handle(keyEvent(49, " "), client: client)
            }
        }
        expect(selected == [.english, .korean, .auto], "Mode feedback must track shortcut choices")
        for flags: NSEvent.ModifierFlags in [[.command, .shift], [.control, .shift, .option], [.shift]] {
            expect(!coordinator.handle(keyEvent(99, "", flags: flags), client: client), "Other shortcuts must pass through")
        }
        expect(coordinator.mode == .auto, "Unrelated modifiers must not change the language mode")

        let secure = InputCoordinator(secureInputEnabled: { true })
        let password = PresentationClient()
        for code: UInt16 in [49, 99, 120, 122] {
            expect(!secure.handle(keyEvent(code, "", flags: [.control, .shift]), client: password), "Secure fields must bypass manual shortcuts too")
        }
        expect(password.commits.isEmpty && password.lastMarked == nil && secure.mode == .auto, "Manual commands must never write into a password field")
    }

    private static func verifySecureInput() {
        for mode in InputMode.allCases {
            var secure = false
            let coordinator = InputCoordinator(mode: mode, secureInputEnabled: { secure })
            let previous = PresentationClient()
            _ = coordinator.handle(keyEvent(15, "r"), client: previous)
            _ = coordinator.handle(keyEvent(40, "k"), client: previous)
            expect(!coordinator.markedText.isEmpty, "Prepare pending text before entering a password field")
            let password = PresentationClient()
            secure = true
            for (code, text) in [(UInt16(0), "a"), (0, "A"), (51, "\u{7f}"), (49, " "), (36, "\r")] {
                expect(!coordinator.handle(keyEvent(code, text), client: password), "Secure input must forward each original key")
            }
            coordinator.finish(client: password)
            coordinator.changeMode(.korean, client: password)
            expect(coordinator.markedText.isEmpty, "Secure input must discard pending text and context")
            expect(password.lastMarked == nil && password.commits.isEmpty, "No marked, empty replacement or committed text may reach a password field")
            secure = false
            coordinator.finish(client: password)
            expect(password.commits.isEmpty, "Leaving secure input must not flush an old composition")
            expect(coordinator.mode == mode, "Secure input must preserve the user's normal typing mode")
            _ = coordinator.handle(keyEvent(0, "a"), client: password)
            expect(coordinator.markedText == (mode == .korean ? "ㅁ" : "a"), "Normal input must resume with a fresh session")
        }

        var secure = false
        let coordinator = InputCoordinator(mode: .korean, secureInputEnabled: { secure })
        let transition = SecureTransitionClient { secure = true }
        for _ in 0..<64 { _ = coordinator.handle(keyEvent(15, "r"), client: transition) }
        transition.secureOnCommit = true
        expect(!coordinator.handle(keyEvent(40, "k"), client: transition), "A key whose new preedit was suppressed must be forwarded")
        expect(transition.commits == 1 && transition.marks == 64, "A secure transition during commit must suppress the following preedit")
        expect(coordinator.markedText.isEmpty, "A secure transition during an edit must discard the remaining state")

        // Exercise the real OS query and NSSecureTextField without showing a
        // window or reading any user's password. Balance only our own enable.
        expect(EnableSecureEventInput() == noErr, "Enable the test's secure input scope")
        defer { DisableSecureEventInput() }
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 300, height: 80), styleMask: [.titled], backing: .buffered, defer: false)
        let password = NSSecureTextField(frame: NSRect(x: 0, y: 0, width: 280, height: 30))
        window.contentView!.addSubview(password)
        password.selectText(nil)
        guard let editor = password.currentEditor() as? NSTextView else { fatalError("Missing secure field editor") }
        defer { window.makeFirstResponder(nil) }
        let systemCoordinator = InputCoordinator(mode: .korean)
        let client = SecureFieldClient(editor)
        let keys: [(UInt16, String)] = [(15, "r"), (40, "k"), (0, "A"), (18, "1"), (18, "!")]
        var expected = ""
        for (code, text) in keys {
            expect(!systemCoordinator.handle(keyEvent(code, text), client: client), "The production OS security query must bypass composition")
            editor.insertText(text, replacementRange: NSRange(location: NSNotFound, length: 0))
            expected += text
            expect(editor.string == expected && !editor.hasMarkedText(), "The secure field must receive exactly one original character per key")
        }
        expect(!systemCoordinator.handle(keyEvent(51, "\u{7f}"), client: client), "Secure Backspace must reach the field")
        editor.deleteBackward(nil)
        systemCoordinator.finish(client: client)
        expect(editor.string == String(expected.dropLast()), "Secure deletion must remove only one original character")
        expect(client.writes == 0, "The input method must never write into the secure field")
    }

    private static func verifyCompetingShortReadings(_ input: TypingView) {
        let korean: [(UInt16, String)] = [(5, "g"), (40, "k"), (1, "s"), (15, "r"), (46, "m"), (3, "f"), (49, " ")]
        let english: [(UInt16, String)] = [(4, "h"), (14, "e"), (37, "l"), (37, "l"), (31, "o"), (49, " ")]
        let contexts = [(0, korean, ""), (1, korean, "한글 "), (1, english, "hello ")]
        let expressions: [(String, [(UInt16, String)])] = [
            ("vs", [(9, "v"), (1, "s")]),
            ("cs", [(8, "c"), (1, "s")]),
            ("tv", [(17, "t"), (9, "v")]),
            ("sg", [(1, "s"), (5, "g")]),
            ("ㄹㅇ", [(3, "f"), (2, "d")]),
            ("ㅇㅈ", [(2, "d"), (13, "w")]),
            ("ㄱㅅ", [(15, "r"), (17, "t")]),
            ("ㄱㅅㄱㅅ", [(15, "r"), (17, "t"), (15, "r"), (17, "t")]),
            ("ㄹㅇㅋㅋ", [(3, "f"), (2, "d"), (6, "z"), (6, "z")]),
        ]
        for (count, context, before) in contexts {
            for (text, keys) in expressions {
                reset(input)
                for _ in 0..<count { press(input, context) }
                press(input, keys)
                let selected = text
                let expected = before + selected
                expect(input.string == expected && input.hasMarkedText(), "Initial reading: \(expected), actual: \(input.string)")
                press(input, [(51, "\u{7f}")])
                expect(input.string == before + String(selected.dropLast()), "Delete one initial, including a compound-coda pair")
                press(input, [keys.last!])
                expect(input.string == expected, "Retyping must restore the selected reading")
                press(input, [(49, " ")])
                input.coordinator.finish(client: input)
                expect(input.string == expected + " " && !input.hasMarkedText(), "Initial expression must commit only once")
            }
        }
    }

    private static func verifyConnectedEnglishSpelling(_ input: TypingView) {
        let korean: [(UInt16, String)] = [(5, "g"), (40, "k"), (1, "s"), (15, "r"), (46, "m"), (3, "f"), (49, " ")]
        let english: [(UInt16, String)] = [(4, "h"), (14, "e"), (37, "l"), (37, "l"), (31, "o"), (49, " ")]
        let contexts = [(0, korean, ""), (4, korean, "한글 "), (4, english, "hello ")]
        let words: [(String, [(UInt16, String)])] = [
            ("fallback", [(3, "f"), (0, "a"), (37, "l"), (37, "l"), (11, "b"), (0, "a"), (8, "c"), (40, "k")]),
            ("rollback", [(15, "r"), (31, "o"), (37, "l"), (37, "l"), (11, "b"), (0, "a"), (8, "c"), (40, "k")]),
            ("switch", [(1, "s"), (13, "w"), (34, "i"), (17, "t"), (8, "c"), (4, "h")]),
            ("macos", [(46, "m"), (0, "a"), (8, "c"), (31, "o"), (1, "s")]),
            ("tacos", [(17, "t"), (0, "a"), (8, "c"), (31, "o"), (1, "s")]),
            ("query", [(12, "q"), (32, "u"), (14, "e"), (15, "r"), (16, "y")]),
            ("angry", [(0, "a"), (45, "n"), (5, "g"), (15, "r"), (16, "y")]),
            ("graphql", [(5, "g"), (15, "r"), (0, "a"), (35, "p"), (4, "h"), (12, "q"), (37, "l")]),
            ("florpdb", [(3, "f"), (37, "l"), (31, "o"), (15, "r"), (35, "p"), (2, "d"), (11, "b")]),
        ]
        let suffixes: [(String, [(UInt16, String)])] = [
            ("", []), ("가", [(15, "r"), (40, "k")]),
            ("갑사", [(15, "r"), (40, "k"), (12, "q"), (17, "t"), (40, "k")]),
        ]
        for (count, context, contextText) in contexts {
            for (word, keys) in words {
                for (suffix, suffixKeys) in suffixes {
                    reset(input)
                    for _ in 0..<count { press(input, context) }
                    let before = String(repeating: contextText, count: count)
                    var partial = ""
                    for key in keys {
                        press(input, [key])
                        partial += key.1
                        if ["fallback", "macos", "tacos"].contains(word) {
                            expect(input.string == before + partial, "\(word) must stay Latin at every keystroke")
                        }
                    }
                    expect(input.string == before + word, "The English word must remain intact before its suffix")
                    if suffix.isEmpty {
                        press(input, [(51, "\u{7f}")])
                        expect(input.string == before + String(word.dropLast()), "English deletion must remove one letter")
                        press(input, [keys.last!])
                        expect(input.string == before + word, "Retyping must restore the English word")
                    }
                    press(input, suffixKeys)
                    let expected = before + word + suffix
                    expect(input.string == expected && input.hasMarkedText(), "Connected spelling: \(expected), actual: \(input.string)")
                    if suffix == "갑사" {
                        press(input, [(51, "\u{7f}")])
                        expect(input.string == before + word + "갑ㅅ", "Korean deletion must preserve the English spelling")
                        press(input, [(40, "k")])
                    }
                    press(input, [(49, " ")])
                    expect(input.string == expected + " " && !input.hasMarkedText(), "Space must commit the full word once")
                }
            }
        }
    }

    private static func verifyMarkedTextPresentation() {
        let coordinator = InputCoordinator(mode: .korean)
        let client = PresentationClient()
        let steps: [(UInt16, String, String)] = [
            (5, "g", "ㅎ"), (40, "k", "하"), (1, "s", "한"),
            (15, "r", "한ㄱ"), (46, "m", "한그"), (3, "f", "한글"),
            (51, "\u{7f}", "한그"), (51, "\u{7f}", "한ㄱ"),
            (51, "\u{7f}", "한"), (51, "\u{7f}", ""),
        ]
        for (code, key, expected) in steps {
            expect(coordinator.handle(keyEvent(code, key), client: client), "Composition key must be consumed")
            guard let marked = client.lastMarked else { fatalError("Missing presentation") }
            expect(marked.attributedString.string == expected, "Styling must preserve every composition step")
            expect(marked.selectionRange == NSRange(location: expected.utf16.count, length: 0), "Composition caret must remain at the UTF-16 end")
            marked.attributedString.enumerateAttributes(in: NSRange(location: 0, length: marked.attributedString.length)) { attributes, _, _ in
                expect(attributes[.underlineStyle] as? Int == NSUnderlineStyle.single.rawValue, "Every composing character needs an explicit underline")
                expect(attributes[.font] == nil && attributes[.foregroundColor] == nil && attributes[.backgroundColor] == nil, "Composition must inherit the host font and colors")
            }
        }
        expect(client.commits.isEmpty, "Presentation must not commit or split a token while it is edited")
        _ = coordinator.handle(keyEvent(15, "r"), client: client)
        _ = coordinator.handle(keyEvent(40, "k"), client: client)
        coordinator.finish(client: client)
        coordinator.finish(client: client)
        expect(client.commits == ["가"], "Commit must remove composition styling without duplicating text")
    }

    private static func verifyEnglishFirstBoundaries(_ input: TypingView) {
        let korean: [(UInt16, String)] = [(5, "g"), (40, "k"), (1, "s"), (15, "r"), (46, "m"), (3, "f"), (49, " ")]
        let english: [(UInt16, String)] = [(4, "h"), (14, "e"), (37, "l"), (37, "l"), (31, "o"), (49, " ")]
        let contexts = [(0, korean, ""), (4, korean, "한글 "), (4, english, "hello ")]
        let prefixes: [([(UInt16, String)], String)] = [
            ([(0, "a"), (34, "i")], "ai"),
            ([(0, "A"), (34, "I")], "AI"),
            ([(12, "q"), (34, "i")], "qi"),
            ([(34, "i"), (31, "o")], "io"),
            ([(4, "h"), (14, "e"), (37, "l"), (37, "l"), (31, "o")], "hello"),
        ]
        let suffixes: [([(UInt16, String)], String)] = [
            ([(15, "r"), (4, "h")], "고"),
            ([(15, "r"), (40, "k")], "가"),
            ([(3, "f"), (4, "h")], "로"),
            ([(2, "d"), (35, "p"), (15, "r"), (35, "p")], "에게"),
            ([(15, "r"), (40, "k"), (12, "q"), (17, "t"), (40, "k")], "갑사"),
        ]
        for (count, context, contextText) in contexts {
            for (prefixKeys, prefix) in prefixes {
                for (suffixKeys, suffix) in suffixes {
                    reset(input)
                    for _ in 0..<count { press(input, context) }
                    let before = String(repeating: contextText, count: count) + prefix
                    let expected = before + suffix
                    press(input, prefixKeys + suffixKeys)
                    expect(input.string == expected && input.hasMarkedText(), "Mixed boundary: \(expected), actual: \(input.string)")
                    if suffix == "갑사" {
                        press(input, [(51, "\u{7f}")])
                        expect(input.string == before + "갑ㅅ", "Mixed deletion must preserve the settled syllable")
                        press(input, [(40, "k")])
                        expect(input.string == expected, "Mixed retyping must restore the suffix")
                    }
                    press(input, [(49, " ")])
                    expect(input.string == expected + " " && !input.hasMarkedText(), "Mixed boundary must commit once: \(expected)")
                }
            }
        }
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

    private static func keyEvent(_ code: UInt16, _ text: String, flags: NSEvent.ModifierFlags? = nil, repeatKey: Bool = false) -> NSEvent {
        let modifiers: NSEvent.ModifierFlags = flags ?? (text != text.lowercased() ? [.shift] : [])
        return NSEvent.keyEvent(with: .keyDown, location: .zero, modifierFlags: modifiers, timestamp: 0, windowNumber: 0, context: nil, characters: text, charactersIgnoringModifiers: text, isARepeat: repeatKey, keyCode: code)!
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
    func mark(_ text: MarkedText) { coordinator.finish(client: self) }
    func commit(_ text: String) {
        commits.append(text)
        coordinator.finish(client: self)
    }
}

@MainActor
private final class PresentationClient: CompositionClient {
    var lastMarked: MarkedText?
    var commits: [String] = []
    var onMark: (() -> Void)?

    func mark(_ text: MarkedText) {
        lastMarked = text
        onMark?()
    }
    func commit(_ text: String) { commits.append(text) }
}

@MainActor
private final class SecureTransitionClient: CompositionClient {
    var marks = 0
    var commits = 0
    var secureOnCommit = false
    private let enterSecureInput: () -> Void

    init(enterSecureInput: @escaping () -> Void) { self.enterSecureInput = enterSecureInput }
    func mark(_ text: MarkedText) { marks += 1 }
    func commit(_ text: String) {
        commits += 1
        if secureOnCommit { enterSecureInput() }
    }
}

@MainActor
private final class SecureFieldClient: CompositionClient {
    let editor: NSTextView
    var writes = 0

    init(_ editor: NSTextView) { self.editor = editor }
    func mark(_ text: MarkedText) {
        writes += 1
        editor.setMarkedText(text.attributedString, selectedRange: text.selectionRange, replacementRange: NSRange(location: NSNotFound, length: 0))
    }
    func commit(_ text: String) {
        writes += 1
        editor.insertText(text, replacementRange: NSRange(location: NSNotFound, length: 0))
    }
}
