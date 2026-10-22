#include "key_input.h"

namespace amu {
KeyInput translateKey(WPARAM key, Modifiers modifiers) {
    if (modifiers.control && modifiers.shift && !modifiers.alt && !modifiers.windows && key >= VK_F1 && key <= VK_F3) {
        return {KeyAction::mode, {3, static_cast<uint32_t>(key - VK_F1)}};
    }
    if (key == VK_SHIFT || key == VK_LSHIFT || key == VK_RSHIFT) return {KeyAction::ignore, {}};
    if (modifiers.control || modifiers.alt || modifiers.windows) return {KeyAction::finish, {2, 0}};
    if (key >= 'A' && key <= 'Z') {
        bool uppercase = modifiers.shift != modifiers.capsLock;
        return {KeyAction::input, {0, static_cast<uint32_t>(uppercase ? key : key + ('a' - 'A'))}};
    }
    if (key == VK_BACK) return {KeyAction::input, {1, 0}};
    if (key == VK_SPACE) return {KeyAction::boundary, {4, 0}};
    return {KeyAction::finish, {2, 0}};
}
Modifiers keyboardModifiers() {
    auto down = [](int key) { return (GetKeyState(key) & 0x8000) != 0; };
    return {down(VK_SHIFT), down(VK_CONTROL), down(VK_MENU), down(VK_LWIN) || down(VK_RWIN), (GetKeyState(VK_CAPITAL) & 1) != 0};
}
}
