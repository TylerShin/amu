import AppKit
import InputMethodKit

@objc(AmuInputController)
@MainActor
final class AmuInputController: IMKInputController {
    private let coordinator = InputCoordinator(mode: InputPreferences.loadMode())

    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let event, let input = sender as? IMKTextInput else { return false }
        return coordinator.handle(event, client: InputMethodClient(input))
    }

    override func commitComposition(_ sender: Any!) {
        guard let input = sender as? IMKTextInput else { return }
        coordinator.finish(client: InputMethodClient(input))
    }

    override func activateServer(_ sender: Any!) {
        coordinator.reset()
        if let input = sender as? IMKTextInput {
            coordinator.changeMode(InputPreferences.loadMode(), client: InputMethodClient(input))
        }
        super.activateServer(sender)
    }

    override func deactivateServer(_ sender: Any!) {
        commitComposition(sender)
        coordinator.reset()
        super.deactivateServer(sender)
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
            let item = NSMenuItem(title: mode.title, action: #selector(selectMode(_:)), keyEquivalent: "")
            item.tag = Int(mode.rawValue)
            item.state = coordinator.mode == mode ? .on : .off
            menu.addItem(item)
        }
        return menu
    }

    @objc private func selectMode(_ sender: Any?) {
        let item = (sender as? NSMenuItem) ?? (sender as? [String: Any])?[kIMKCommandMenuItemName] as? NSMenuItem
        guard let item, let mode = InputMode(rawValue: UInt32(item.tag)), let input = client() else { return }
        coordinator.changeMode(mode, client: InputMethodClient(input))
        InputPreferences.saveMode(mode)
    }
}

@MainActor
private final class InputMethodClient: CompositionClient {
    private let input: IMKTextInput

    init(_ input: IMKTextInput) { self.input = input }

    func mark(_ text: String) {
        input.setMarkedText(text, selectionRange: NSRange(location: text.utf16.count, length: 0), replacementRange: NSRange(location: NSNotFound, length: 0))
    }

    func commit(_ text: String) {
        input.insertText(text, replacementRange: NSRange(location: NSNotFound, length: 0))
    }
}
