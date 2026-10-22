import AppKit

@MainActor
final class RuntimeStatus: NSObject {
    override init() {
        super.init()
        DistributedNotificationCenter.default().addObserver(
            self, selector: #selector(replyWithVersion(_:)), name: AmuRuntimeProtocol.request,
            object: String(ProcessInfo.processInfo.processIdentifier), suspensionBehavior: .deliverImmediately
        )
        DistributedNotificationCenter.default().addObserver(
            self, selector: #selector(stopIfIdle(_:)), name: AmuRuntimeProtocol.stop,
            object: String(ProcessInfo.processInfo.processIdentifier), suspensionBehavior: .deliverImmediately
        )
    }

    @objc private func stopIfIdle(_ notification: Notification) {
        // 설치기의 요청만 검사한다. 로그아웃 등 OS의 정상 종료를 막지 않는다.
        guard !AmuRuntimeProtocol.isInputSourceSelected, !AmuInputController.hasMarkedText else { return }
        NSApplication.shared.terminate(nil)
    }

    @objc private func replyWithVersion(_ notification: Notification) {
        guard let token = notification.userInfo?["token"] as? String else { return }
        DistributedNotificationCenter.default().postNotificationName(
            AmuRuntimeProtocol.response, object: token,
            userInfo: ["pid": ProcessInfo.processInfo.processIdentifier, "build": AmuBuild.json], deliverImmediately: true
        )
    }
}
