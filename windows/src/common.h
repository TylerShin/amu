#pragma once
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <msctf.h>
#include <wrl/client.h>
#include <atomic>
#include <new>
#include <string>

namespace amu {
using Microsoft::WRL::ComPtr;
inline constexpr GUID ServiceId = {0x991abf87,0x12a1,0x4f5c,{0xa9,0xcc,0x80,0x07,0x24,0xad,0x3a,0xf3}};
inline constexpr GUID ProfileId = {0x19cce201,0xbaf0,0x4a9d,{0xa1,0x71,0xf3,0x38,0x4b,0xbb,0x4f,0x69}};
inline constexpr GUID UnderlineId = {0x7c5f745f,0x3496,0x40df,{0xb9,0x6a,0x81,0x38,0xd7,0x2a,0x8f,0x21}};
inline constexpr LANGID LanguageId = 0x0412;
inline std::atomic<long> moduleObjects{0};
inline HMODULE moduleHandle = nullptr;

// Every COM object, including queued edit sessions, keeps the DLL loaded.
class Lifetime {
public:
    Lifetime() { ++moduleObjects; }
    virtual ~Lifetime() { --moduleObjects; }
    ULONG retain() { return ++references_; }
    ULONG release() { auto count = --references_; if (!count) delete this; return count; }
private:
    std::atomic<ULONG> references_{1};
};
}
