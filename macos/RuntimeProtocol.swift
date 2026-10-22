import Carbon
import Foundation

enum AmuRuntimeProtocol {
    static let bundleIdentifier = "dev.amu.inputmethod.Amu"
    static let request = Notification.Name("dev.amu.runtime.version.request")
    static let response = Notification.Name("dev.amu.runtime.version.response")
    static let stop = Notification.Name("dev.amu.runtime.stop-if-idle")

    static var isInputSourceSelected: Bool {
        let source = TISCopyCurrentKeyboardInputSource().takeRetainedValue()
        guard let pointer = TISGetInputSourceProperty(source, kTISPropertyInputSourceID) else { return true }
        let identifier = Unmanaged<CFString>.fromOpaque(pointer).takeUnretainedValue() as String
        return identifier == bundleIdentifier || identifier.hasPrefix(bundleIdentifier + ".")
    }
}
