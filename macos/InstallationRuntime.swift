import AppKit

@MainActor
enum InstallationRuntime {
    static func stop(bundle: URL) throws {
        guard !AmuRuntimeProtocol.isInputSourceSelected else {
            throw InstallationError("Amu가 선택되어 있습니다. 한국어 또는 ABC로 전환한 뒤 다시 설치하세요. 기존 앱은 유지합니다.")
        }
        for application in try applications(at: bundle) {
            guard !AmuRuntimeProtocol.isInputSourceSelected else { throw InstallationError("입력 소스가 Amu로 바뀌어 종료를 중단했습니다.") }
            let build = VersionReply().read(pid: application.processIdentifier)
            if let version = build?["runtimeProtocol"] as? Int {
                guard version == 1 else { throw InstallationError("지원하지 않는 설치 프로토콜입니다. 설치 도구를 업데이트하세요.") }
                DistributedNotificationCenter.default().postNotificationName(
                    AmuRuntimeProtocol.stop, object: String(application.processIdentifier), userInfo: nil, deliverImmediately: true
                )
            } else {
                // 버전 응답이 없던 초기 앱에만 정상 종료 요청을 사용한다.
                let metadata = bundle.appendingPathComponent("Contents/Resources/build.json")
                if build == nil && FileManager.default.fileExists(atPath: metadata.path) {
                    throw InstallationError("실행 중인 앱이 응답하지 않아 종료하지 않았습니다.")
                }
                guard !AmuRuntimeProtocol.isInputSourceSelected else { throw InstallationError("입력 소스가 Amu로 바뀌어 종료를 중단했습니다.") }
                _ = application.terminate()
            }
            let deadline = Date().addingTimeInterval(5)
            while !application.isTerminated && Date() < deadline {
                RunLoop.current.run(until: Date().addingTimeInterval(0.05))
            }
            guard application.isTerminated else { throw InstallationError("Amu가 종료를 거부했습니다. 조합을 마치고 다른 입력기로 전환한 뒤 다시 시도하세요. 강제 종료하지 않았습니다.") }
        }
        guard try applications(at: bundle).isEmpty else { throw InstallationError("Amu가 다시 실행되어 교체를 중단했습니다.") }
    }

    static func verify(bundle: URL, expected: String?) throws {
        let deadline = Date().addingTimeInterval(8)
        while Date() < deadline {
            let running = try applications(at: bundle)
            if running.count == 1, let application = running.first,
               let build = VersionReply().read(pid: application.processIdentifier) {
                guard let identifier = build["identifier"] as? String else { throw InstallationError("실행 중인 앱의 버전 응답이 올바르지 않습니다.") }
                if let expected, identifier != expected { throw InstallationError("실행 버전 불일치: 예상 \(expected), 실제 \(identifier)") }
                guard !application.isTerminated else { continue }
                let result: [String: Any] = ["status": "running", "pid": application.processIdentifier, "build": build]
                let data = try JSONSerialization.data(withJSONObject: result, options: [.sortedKeys])
                print(String(decoding: data, as: UTF8.self))
                return
            }
            RunLoop.current.run(until: Date().addingTimeInterval(0.1))
        }
        throw InstallationError("실행 중인 Amu에서 버전 응답을 받지 못했습니다. 적용 완료로 처리하지 않았습니다.")
    }

    private static func applications(at bundle: URL) throws -> [NSRunningApplication] {
        let running = NSRunningApplication.runningApplications(withBundleIdentifier: AmuRuntimeProtocol.bundleIdentifier)
        for application in running {
            guard application.bundleURL?.standardizedFileURL == bundle else {
                throw InstallationError("다른 경로의 Amu가 실행 중입니다: \(application.bundleURL?.path ?? "unknown"). 해당 앱은 종료하지 않았습니다.")
            }
        }
        return running
    }
}

@MainActor
private final class VersionReply: NSObject {
    private var build: [String: Any]?
    private var pid: pid_t = 0

    func read(pid: pid_t) -> [String: Any]? {
        self.pid = pid
        let token = UUID().uuidString
        let center = DistributedNotificationCenter.default()
        center.addObserver(self, selector: #selector(receive(_:)), name: AmuRuntimeProtocol.response, object: token, suspensionBehavior: .deliverImmediately)
        defer { center.removeObserver(self) }
        center.postNotificationName(AmuRuntimeProtocol.request, object: String(pid), userInfo: ["token": token], deliverImmediately: true)
        let deadline = Date().addingTimeInterval(0.5)
        while build == nil && Date() < deadline { RunLoop.current.run(until: Date().addingTimeInterval(0.02)) }
        return build
    }

    @objc private func receive(_ notification: Notification) {
        guard let sender = notification.userInfo?["pid"] as? NSNumber, sender.int32Value == pid,
              let json = notification.userInfo?["build"] as? String,
              let data = json.data(using: .utf8),
              let value = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else { return }
        build = value
    }
}

private struct InstallationError: Error, CustomStringConvertible {
    let description: String
    init(_ description: String) { self.description = description }
}
