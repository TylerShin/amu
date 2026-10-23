import AppKit
import Carbon

@MainActor
protocol CompositionClient: AnyObject {
    func mark(_ text: MarkedText)
    func commit(_ text: String)
}

@MainActor
final class InputCoordinator {
    private let secureInputEnabled: () -> Bool
    private var session: CoreSession?
    private var lastMarked = ""
    private var applying = false
    private var finishRequested = false
    private(set) var mode: InputMode = .auto
    var modeDidChange: ((InputMode) -> Void)?

    var markedText: String { discardIfSecureInput() ? "" : lastMarked }

    init(mode: InputMode = .auto, secureInputEnabled: @escaping () -> Bool = { IsSecureEventInputEnabled() }) {
        self.mode = mode
        self.secureInputEnabled = secureInputEnabled
        session = try? CoreSession(mode: mode)
    }

    func handle(_ event: NSEvent, client: CompositionClient) -> Bool {
        guard !discardIfSecureInput() else { return false }
        guard event.type == .keyDown, !applying, let session else { return false }
        let key = KeyTranslator.translate(event)
        if case .changeMode(let mode) = key {
            return event.isARepeat || changeMode(mode, client: client)
        }
        if case .cycleReading = key, event.isARepeat { return true }
        do {
            let update = try session.process(key)
            return apply(update, client: client)
        } catch {
            recover(client: client)
            return false
        }
    }

    func finish(client: CompositionClient) {
        guard !discardIfSecureInput() else { return }
        if applying { finishRequested = true; return }
        guard let session else { return }
        do { apply(try session.process(.finish), client: client) }
        catch { recover(client: client) }
    }

    @discardableResult
    func changeMode(_ mode: InputMode, client: CompositionClient) -> Bool {
        guard !discardIfSecureInput() else { return false }
        guard !applying, let session else { return false }
        do {
            let update = try session.process(.changeMode(mode))
            self.mode = mode
            apply(update, client: client)
            modeDidChange?(mode)
            return true
        } catch {
            recover(client: client)
            return false
        }
    }

    func reset() {
        session?.reset()
        lastMarked = ""
        finishRequested = false
    }

    @discardableResult
    private func apply(_ update: CompositionUpdate, client: CompositionClient) -> Bool {
        guard !discardIfSecureInput() else { return false }
        let previouslyMarked = lastMarked
        lastMarked = update.preedit
        applying = true
        if !update.commit.isEmpty { client.commit(update.commit) }
        if discardIfSecureInput() {
            applying = false
            return false
        }
        if !update.preedit.isEmpty || (!previouslyMarked.isEmpty && update.commit.isEmpty) {
            client.mark(MarkedText(update.preedit))
        }
        applying = false
        if finishRequested {
            finishRequested = false
            finish(client: client)
        }
        return update.consumed
    }

    private func recover(client: CompositionClient) {
        guard !discardIfSecureInput() else { return }
        let preserved = lastMarked
        session = nil
        lastMarked = ""
        finishRequested = false
        if !preserved.isEmpty { client.commit(preserved) }
        NSLog("Amu core unavailable; passing subsequent keys to the application")
    }

    private func discardIfSecureInput() -> Bool {
        guard secureInputEnabled() else { return false }
        // The focused client may already be a password field. Do not send it
        // old preedit, an empty replacement, or even a finishing callback.
        reset()
        return true
    }
}
