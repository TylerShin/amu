#include <windows.h>
#include <delayimp.h>
#include <cstring>
#include <string>

namespace {
FARPROC WINAPI loadAdjacentLibrary(unsigned notification, PDelayLoadInfo info) {
    if (notification != dliNotePreLoadLibrary || std::strcmp(info->szDll, "hangul.dll") != 0) return nullptr;
    HMODULE owner = nullptr;
    if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        reinterpret_cast<LPCWSTR>(&loadAdjacentLibrary), &owner)) return nullptr;
    wchar_t path[32768]{};
    DWORD length = GetModuleFileNameW(owner, path, ARRAYSIZE(path));
    if (!length || length >= ARRAYSIZE(path)) return nullptr;
    std::wstring library(path, length);
    library.resize(library.find_last_of(L"\\/") + 1);
    library += L"hangul.dll";
    auto loaded = LoadLibraryExW(library.c_str(), nullptr, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32);
    // Do not fall back to searching an unrelated application's working directory.
    if (!loaded) RaiseException(VcppException(ERROR_SEVERITY_ERROR, ERROR_MOD_NOT_FOUND), 0, 0, nullptr);
    return reinterpret_cast<FARPROC>(loaded);
}
}
extern "C" const PfnDliHook __pfnDliNotifyHook2 = loadAdjacentLibrary;
