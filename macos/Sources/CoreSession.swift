import Foundation

enum InputMode: UInt32, CaseIterable {
    case auto = 0, korean = 1, english = 2

    var title: String {
        switch self {
        case .auto: return "자동 판독 · 실험"
        case .korean: return "한글 두벌식"
        case .english: return "영문"
        }
    }
}

struct CompositionUpdate {
    let consumed: Bool
    let commit: String
    let preedit: String
}

enum CoreOperation {
    case letter(UInt8), backspace, finish, changeMode(InputMode), wordBoundary

    var abiValues: (kind: UInt32, value: UInt32) {
        switch self {
        case .letter(let ascii): return (0, UInt32(ascii))
        case .backspace: return (1, 0)
        case .finish: return (2, 0)
        case .changeMode(let mode): return (3, mode.rawValue)
        case .wordBoundary: return (4, 0)
        }
    }
}

enum CoreError: Error { case unavailable, operationFailed(UInt32), invalidOutput }

@MainActor
final class CoreSession {
    private let handle: OpaquePointer

    init(mode: InputMode = .auto) throws {
        guard let handle = amu_session_new(mode.rawValue) else { throw CoreError.unavailable }
        self.handle = handle
    }

    deinit { amu_session_free(handle) }

    func process(_ operation: CoreOperation) throws -> CompositionUpdate {
        var output = AmuOutput()
        let arguments = operation.abiValues
        let status = amu_session_process(handle, arguments.kind, arguments.value, &output)
        guard status == 0 else { throw CoreError.operationFailed(status) }
        guard output.commit_len <= 1024, output.preedit_len <= 1024 else { throw CoreError.invalidOutput }
        let commit = withUnsafeBytes(of: output.commit) { String(decoding: $0.prefix(Int(output.commit_len)), as: UTF8.self) }
        let preedit = withUnsafeBytes(of: output.preedit) { String(decoding: $0.prefix(Int(output.preedit_len)), as: UTF8.self) }
        return CompositionUpdate(consumed: output.consumed != 0, commit: commit, preedit: preedit)
    }

    func reset() { _ = amu_session_reset(handle) }
}
