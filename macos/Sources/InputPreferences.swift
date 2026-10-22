import Foundation

enum InputPreferences {
    static func loadMode() -> InputMode {
        let value = UserDefaults.standard.integer(forKey: "inputMode")
        guard let rawValue = UInt32(exactly: value) else { return .auto }
        return InputMode(rawValue: rawValue) ?? .auto
    }

    static func saveMode(_ mode: InputMode) {
        UserDefaults.standard.set(mode.rawValue, forKey: "inputMode")
    }
}
