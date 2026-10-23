import AppKit
import InputMethodKit

@objc(AmuInputController)
@MainActor
final class AmuInputController: IMKInputController {
    private static weak var activeController: AmuInputController?
    static var hasMarkedText: Bool { !(activeController?.coordinator.markedText.isEmpty ?? true) }
    private let coordinator = InputCoordinator()
    private let modeIndicator = ModeIndicator()

    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let event, let input = sender as? IMKTextInput else { return false }
        return coordinator.handle(event, client: InputMethodClient(input))
    }

    override func commitComposition(_ sender: Any!) {
        guard let input = sender as? IMKTextInput else { return }
        coordinator.finish(client: InputMethodClient(input))
    }

    override func activateServer(_ sender: Any!) {
        Self.activeController = self
        coordinator.reset()
        if let input = sender as? IMKTextInput {
            coordinator.changeMode(.auto, client: InputMethodClient(input))
        }
        coordinator.modeDidChange = { [weak self] mode in self?.modeIndicator.show(mode) }
        super.activateServer(sender)
    }

    override func deactivateServer(_ sender: Any!) {
        coordinator.modeDidChange = nil
        modeIndicator.hide()
        commitComposition(sender)
        coordinator.reset()
        super.deactivateServer(sender)
        if Self.activeController === self { Self.activeController = nil }
    }

    override func recognizedEvents(_ sender: Any!) -> Int { Int(NSEvent.EventTypeMask.keyDown.rawValue) }

    override func composedString(_ sender: Any!) -> Any! { coordinator.markedText }

    override func originalString(_ sender: Any!) -> NSAttributedString! {
        NSAttributedString(string: coordinator.markedText)
    }

    override func cancelComposition() {
        guard let input = client() else { return }
        coordinator.finish(client: InputMethodClient(input))
    }

    override func menu() -> NSMenu! {
        let menu = NSMenu(title: "Amu")
        for mode in InputMode.allCases {
            let shortcut = ["⌃⇧F1", "⌃⇧F2", "⌃⇧F3"][Int(mode.rawValue)]
            let item = NSMenuItem(title: "\(mode.title) · \(shortcut)", action: #selector(selectMode(_:)), keyEquivalent: "")
            item.tag = Int(mode.rawValue)
            item.state = coordinator.mode == mode ? .on : .off
            menu.addItem(item)
        }
        menu.addItem(.separator())
        let help = NSMenuItem(title: "현재 단어 바꾸기: ⌃⇧Space (자동 모드)", action: nil, keyEquivalent: "")
        menu.addItem(help)
        menu.addItem(NSMenuItem(title: "입력창을 떠나면 자동 판독으로 돌아갑니다", action: nil, keyEquivalent: ""))
        return menu
    }

    @objc private func selectMode(_ sender: Any?) {
        let item = (sender as? NSMenuItem) ?? (sender as? [String: Any])?[kIMKCommandMenuItemName] as? NSMenuItem
        guard let item, let mode = InputMode(rawValue: UInt32(item.tag)), let input = client() else { return }
        coordinator.changeMode(mode, client: InputMethodClient(input))
    }
}

@MainActor
private final class InputMethodClient: CompositionClient {
    private let input: IMKTextInput

    init(_ input: IMKTextInput) { self.input = input }

    func mark(_ text: MarkedText) {
        input.setMarkedText(text.attributedString, selectionRange: text.selectionRange, replacementRange: NSRange(location: NSNotFound, length: 0))
    }

    func commit(_ text: String) {
        input.insertText(text, replacementRange: NSRange(location: NSNotFound, length: 0))
    }
}
