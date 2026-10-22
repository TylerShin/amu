#include "dependency_loader.h"
#include <delayimp.h>
#include <cstring>
#include <string>

namespace amu {
namespace {
struct CoreLibrary { HMODULE handle; HRESULT status; };
const CoreLibrary& coreLibrary();
CoreLibrary loadAdjacentLibrary();
}
HRESULT prepareCoreLibrary() { return coreLibrary().status; }

namespace {
const CoreLibrary& coreLibrary() {
    static const CoreLibrary library = loadAdjacentLibrary();
    return library;
}
CoreLibrary loadAdjacentLibrary() {
    HMODULE owner = nullptr;
    if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        reinterpret_cast<LPCWSTR>(&loadAdjacentLibrary), &owner)) return {nullptr, HRESULT_FROM_WIN32(GetLastError())};
    wchar_t path[32768]{};
    DWORD length = GetModuleFileNameW(owner, path, ARRAYSIZE(path));
    if (!length || length >= ARRAYSIZE(path)) return {nullptr, E_FAIL};
    std::wstring filename(path, length);
    filename.resize(filename.find_last_of(L"\\/") + 1);
    filename += L"hangul.dll";
    auto loaded = LoadLibraryExW(filename.c_str(), nullptr, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32);
    if (!loaded) return {nullptr, HRESULT_FROM_WIN32(GetLastError())};
    for (const char* symbol : {"hangul_ic_new", "hangul_ic_delete", "hangul_ic_process", "hangul_ic_backspace",
        "hangul_ic_reset", "hangul_ic_get_commit_string", "hangul_ic_get_preedit_string"}) {
        if (!GetProcAddress(loaded, symbol)) { FreeLibrary(loaded); return {nullptr, HRESULT_FROM_WIN32(ERROR_PROC_NOT_FOUND)}; }
    }
    // An in-process IME keeps one validated library for its lifetime. Replacement
    // takes effect in a new host process, matching the immutable installation path.
    return {loaded, S_OK};
}
FARPROC WINAPI delayedLibrary(unsigned notification, PDelayLoadInfo info) {
    if (notification != dliNotePreLoadLibrary || std::strcmp(info->szDll, "hangul.dll") != 0) return nullptr;
    const auto& library = coreLibrary();
    if (!library.handle) RaiseException(VcppException(ERROR_SEVERITY_ERROR, ERROR_MOD_NOT_FOUND), 0, 0, nullptr);
    return reinterpret_cast<FARPROC>(library.handle);
}
}
}
extern "C" const PfnDliHook __pfnDliNotifyHook2 = amu::delayedLibrary;
