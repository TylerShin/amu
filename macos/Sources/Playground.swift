import AppKit

@MainActor
final class Playground: NSObject, NSWindowDelegate {
    private var window: NSWindow?
    private var input: TypingView?

    func show() {
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 780, height: 480), styleMask: [.titled, .closable, .miniaturizable, .resizable], backing: .buffered, defer: false)
        window.title = "Amu · 타이핑 테스트"
        window.delegate = self
        window.isReleasedWhenClosed = false
        let container = NSView(frame: window.contentView!.bounds)
        container.autoresizingMask = [.width, .height]
        let heading = NSTextField(labelWithString: "한/영 전환 없이 입력해 보세요")
        heading.font = .systemFont(ofSize: 22, weight: .semibold)
        heading.frame = NSRect(x: 28, y: 422, width: 600, height: 30)
        heading.autoresizingMask = [.width, .minYMargin]
        container.addSubview(heading)
        let note = NSTextField(labelWithString: "⌃⇧Space 현재 단어 교정 · ⌃⇧F1 자동 · ⌃⇧F2 한글 고정 · ⌃⇧F3 영문 고정")
        note.frame = NSRect(x: 28, y: 388, width: 724, height: 24)
        note.textColor = .secondaryLabelColor
        note.autoresizingMask = [.width, .minYMargin]
        container.addSubview(note)
        let modes = NSSegmentedControl(labels: InputMode.allCases.map(\.title), trackingMode: .selectOne, target: self, action: #selector(changeMode(_:)))
        modes.frame = NSRect(x: 28, y: 346, width: 500, height: 30)
        modes.selectedSegment = 0
        modes.autoresizingMask = [.minYMargin]
        container.addSubview(modes)

        let scroll = NSScrollView(frame: NSRect(x: 28, y: 28, width: 724, height: 300))
        scroll.autoresizingMask = [.width, .height]
        scroll.hasVerticalScroller = true
        scroll.borderType = .bezelBorder
        let input = TypingView(frame: scroll.contentView.bounds)
        input.coordinator.modeDidChange = { [weak modes] mode in modes?.selectedSegment = Int(mode.rawValue) }
        input.font = .systemFont(ofSize: 24)
        input.isRichText = false
        input.isAutomaticQuoteSubstitutionEnabled = false
        input.isAutomaticDashSubstitutionEnabled = false
        input.isAutomaticSpellingCorrectionEnabled = false
        input.textContainerInset = NSSize(width: 18, height: 18)
        input.autoresizingMask = [.width]
        input.textContainer?.widthTracksTextView = true
        input.setAccessibilityIdentifier("amu-typing-area")
        scroll.documentView = input
        container.addSubview(scroll)
        window.contentView = container
        self.input = input
        self.window = window
        window.center()
        window.makeKeyAndOrderFront(nil)
        window.makeFirstResponder(input)
        NSApp.activate(ignoringOtherApps: true)
    }

    func windowWillClose(_ notification: Notification) { NSApp.terminate(nil) }

    func windowDidResignKey(_ notification: Notification) {
        guard let input else { return }
        input.coordinator.changeMode(.auto, client: input)
    }

    @objc private func changeMode(_ sender: NSSegmentedControl) {
        guard let input, let mode = InputMode(rawValue: UInt32(sender.selectedSegment)) else { return }
        input.coordinator.changeMode(mode, client: input)
        window?.makeFirstResponder(input)
    }
}

@MainActor
final class TypingView: NSTextView, CompositionClient {
    let coordinator = InputCoordinator()

    override func keyDown(with event: NSEvent) {
        if !coordinator.handle(event, client: self) { super.keyDown(with: event) }
    }

    override func mouseDown(with event: NSEvent) {
        coordinator.finish(client: self)
        super.mouseDown(with: event)
    }

    override func resignFirstResponder() -> Bool {
        coordinator.finish(client: self)
        return super.resignFirstResponder()
    }

    override func paste(_ sender: Any?) {
        coordinator.finish(client: self)
        super.paste(sender)
    }

    override func selectAll(_ sender: Any?) {
        coordinator.finish(client: self)
        super.selectAll(sender)
    }

    override func cut(_ sender: Any?) {
        coordinator.finish(client: self)
        super.cut(sender)
    }

    func mark(_ text: MarkedText) {
        setMarkedText(text.attributedString, selectedRange: text.selectionRange, replacementRange: NSRange(location: NSNotFound, length: 0))
    }

    func commit(_ text: String) { insertText(text, replacementRange: NSRange(location: NSNotFound, length: 0)) }
}
