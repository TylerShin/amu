import Carbon
import Foundation

@main
struct InputSourceTool {
    @MainActor
    static func main() {
        let parentIdentifier = "dev.amu.inputmethod.Amu"
        let identifier = "dev.amu.inputmethod.Amu.auto"
        let arguments = CommandLine.arguments
        let command = arguments.count > 1 ? arguments[1] : "status"
        if command == "stop" || command == "running" || command == "verify" {
            do {
                guard arguments.count >= 3 else { fail("A bundle path is required") }
                let bundle = URL(fileURLWithPath: arguments[2]).standardizedFileURL
                if command == "stop" {
                    try InstallationRuntime.stop(bundle: bundle)
                } else {
                    let expected = command == "verify" && arguments.count == 4 ? arguments[3] : nil
                    if command == "verify" && expected == nil { fail("verify requires a build identifier") }
                    try InstallationRuntime.verify(bundle: bundle, expected: expected)
                }
                return
            } catch { fail(String(describing: error)) }
        }

        if command == "current" {
            let current = TISCopyCurrentKeyboardInputSource().takeRetainedValue()
            print(stringProperty(current, kTISPropertyInputSourceID))
            exit(0)
        }
        if command == "restore" {
            guard arguments.count == 3, let previous = source(arguments[2]) else { fail("Unknown input source") }
            requireSuccess(TISSelectInputSource(previous), "Restore input source")
            exit(0)
        }
        guard ["status", "register", "enable", "select"].contains(command) else { fail("Unknown command") }
        if command == "register" {
            guard arguments.count == 3 else { fail("register requires an app path") }
            requireSuccess(TISRegisterInputSource(URL(fileURLWithPath: arguments[2]) as CFURL), "Register Amu")
        }
        guard let input = source(identifier) else {
            print("Amu is not visible to macOS yet. Log out and back in, then add Amu in Keyboard → Text Input → Edit.")
            exit(command == "status" ? 0 : 2)
        }
        if command == "register" || command == "enable" {
            if let parent = source(parentIdentifier) { requireSuccess(TISEnableInputSource(parent), "Enable Amu parent") }
            requireSuccess(TISEnableInputSource(input), "Enable Amu mode")
        }
        if command == "select" { requireSuccess(TISSelectInputSource(input), "Select Amu") }
        print("Amu input source registered: \(identifier)")
    }
}

func source(_ identifier: String) -> TISInputSource? {
    let filter = [kTISPropertyInputSourceID as String: identifier] as CFDictionary
    return (TISCreateInputSourceList(filter, true).takeRetainedValue() as! [TISInputSource]).first
}

func stringProperty(_ source: TISInputSource, _ key: CFString) -> String {
    guard let pointer = TISGetInputSourceProperty(source, key) else { return "" }
    return Unmanaged<CFString>.fromOpaque(pointer).takeUnretainedValue() as String
}

func requireSuccess(_ status: OSStatus, _ operation: String) {
    if status != noErr { fail("\(operation) failed: \(status)") }
}

func fail(_ message: String) -> Never {
    FileHandle.standardError.write(Data((message + "\n").utf8))
    exit(1)
}
