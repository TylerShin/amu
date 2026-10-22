#pragma once
#include "core_session.h"

namespace amu {
enum class KeyAction { ignore, input, boundary, finish, mode };
struct KeyInput { KeyAction action; CoreInput core; };
struct Modifiers { bool shift; bool control; bool alt; bool windows; bool capsLock; };
KeyInput translateKey(WPARAM key, Modifiers modifiers);
Modifiers keyboardModifiers();
}
