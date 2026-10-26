import AppKit

@MainActor
final class ModeIndicator {
    private var panel: NSPanel?
    private var dismissal: Task<Void, Never>?

    func show(_ mode: InputMode) {
        hide()
        guard let screen = NSScreen.main else { return }
        let panel = NSPanel(contentRect: NSRect(x: 0, y: 0, width: 320, height: 64),
                            styleMask: [.borderless, .nonactivatingPanel], backing: .buffered, defer: false)
        panel.level = .floating
        panel.hidesOnDeactivate = false
        panel.ignoresMouseEvents = true
        panel.isReleasedWhenClosed = false
        panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
        let label = NSTextField(labelWithString: "Amu · \(mode.title)\n⌃⇧F1 자동 · ⌃⇧F2 한글 · ⌃⇧F3 영문")
        label.alignment = .center
        label.font = .systemFont(ofSize: 13)
        label.frame = NSRect(x: 12, y: 12, width: 296, height: 40)
        panel.contentView?.addSubview(label)
        panel.setFrameOrigin(NSPoint(x: screen.visibleFrame.maxX - 340, y: screen.visibleFrame.maxY - 84))
        panel.orderFrontRegardless()
        self.panel = panel
        dismissal = Task { [weak self] in
            do { try await Task.sleep(nanoseconds: 2_000_000_000) }
            catch { return }
            self?.hide()
        }
    }

    func hide() {
        dismissal?.cancel()
        dismissal = nil
        panel?.orderOut(nil)
        panel = nil
    }
}
