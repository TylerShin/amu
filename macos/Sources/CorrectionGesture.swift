import AppKit
import IOKit.hidsystem

/// A right Shift press/release is a correction only when no other key joined it.
struct CorrectionGesture {
    private var shiftIsDown = false
    private var correctOnRelease = false

    mutating func observe(_ event: NSEvent, canCorrect: Bool) -> Bool {
        let shiftWasDown = shiftIsDown
        shiftIsDown = event.modifierFlags.contains(.shift)
        guard event.type == .flagsChanged, event.keyCode == 60 else {
            correctOnRelease = false
            return false
        }
        // IME transport can omit device-dependent bits. The virtual key code
        // identifies the side; the standard Shift flag identifies press/release.
        if shiftIsDown {
            if !shiftWasDown {
                let modifiers = event.modifierFlags.intersection([.command, .control, .option, .shift, .function])
                let leftShiftIsDown = event.modifierFlags.rawValue & UInt(NX_DEVICELSHIFTKEYMASK) != 0
                correctOnRelease = canCorrect && modifiers == [.shift] && !leftShiftIsDown
            }
            return false
        }
        let correct = shiftWasDown && correctOnRelease && canCorrect
        reset()
        return correct
    }

    mutating func reset() {
        shiftIsDown = false
        correctOnRelease = false
    }
}
