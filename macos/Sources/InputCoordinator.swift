import AppKit

@MainActor
protocol CompositionClient: AnyObject {
    func mark(_ text: MarkedText)
    func commit(_ text: String)
}

@MainActor
final class InputCoordinator {
    private var session: CoreSession?
    private var lastMarked = ""
    private var applying = false
    private var finishRequested = false
    private(set) var mode: InputMode = .auto

    var markedText: String { lastMarked }

    init(mode: InputMode = .auto) {
        self.mode = mode
        session = try? CoreSession(mode: mode)
    }

    func handle(_ event: NSEvent, client: CompositionClient) -> Bool {
        guard event.type == .keyDown, !applying, let session else { return false }
        let key = KeyTranslator.translate(event)
        do {
            let update = try session.process(key)
            apply(update, client: client)
            return update.consumed
        } catch {
            recover(client: client)
            return false
        }
    }

    func finish(client: CompositionClient) {
        if applying { finishRequested = true; return }
        guard let session else { return }
        do { apply(try session.process(.finish), client: client) }
        catch { recover(client: client) }
    }

    func changeMode(_ mode: InputMode, client: CompositionClient) {
        guard !applying, let session else { return }
        do {
            apply(try session.process(.changeMode(mode)), client: client)
            self.mode = mode
        } catch { recover(client: client) }
    }

    func reset() {
        session?.reset()
        lastMarked = ""
        finishRequested = false
    }

    private func apply(_ update: CompositionUpdate, client: CompositionClient) {
        let previouslyMarked = lastMarked
        lastMarked = update.preedit
        applying = true
        if !update.commit.isEmpty { client.commit(update.commit) }
        if !update.preedit.isEmpty || (!previouslyMarked.isEmpty && update.commit.isEmpty) {
            client.mark(MarkedText(update.preedit))
        }
        applying = false
        if finishRequested {
            finishRequested = false
            finish(client: client)
        }
    }

    private func recover(client: CompositionClient) {
        let preserved = lastMarked
        session = nil
        lastMarked = ""
        finishRequested = false
        if !preserved.isEmpty { client.commit(preserved) }
        NSLog("Amu core unavailable; passing subsequent keys to the application")
    }
}
