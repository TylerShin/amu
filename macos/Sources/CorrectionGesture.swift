import AppKit
import IOKit.hidsystem

/// A right Shift press/release is a correction only when no other key joined it.
struct CorrectionGesture {
    private var rightShiftIsDown = false
    private var correctOnRelease = false

    mutating func observe(_ event: NSEvent, canCorrect: Bool) -> Bool {
        guard event.type == .flagsChanged, event.keyCode == 60 else {
            correctOnRelease = false
            return false
        }
        let pressed = event.modifierFlags.rawValue & UInt(NX_DEVICERSHIFTKEYMASK) != 0
        if pressed {
            if !rightShiftIsDown {
                let modifiers = event.modifierFlags.intersection([.command, .control, .option, .shift, .function])
                let leftShiftIsDown = event.modifierFlags.rawValue & UInt(NX_DEVICELSHIFTKEYMASK) != 0
                correctOnRelease = canCorrect && modifiers == [.shift] && !leftShiftIsDown
            }
            rightShiftIsDown = true
            return false
        }
        let correct = rightShiftIsDown && correctOnRelease && canCorrect
        reset()
        return correct
    }

    mutating func reset() {
        rightShiftIsDown = false
        correctOnRelease = false
    }
}
