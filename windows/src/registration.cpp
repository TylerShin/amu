#include "common.h"

namespace amu {
namespace {
constexpr wchar_t ClassKey[] = L"Software\\Classes\\CLSID\\{991ABF87-12A1-4F5C-A9CC-800724AD3AF3}";
const GUID Categories[] = {GUID_TFCAT_TIP_KEYBOARD, GUID_TFCAT_DISPLAYATTRIBUTEPROVIDER,
    GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT, GUID_TFCAT_TIPCAP_COMLESS};

HRESULT writeString(HKEY key, const wchar_t* name, const wchar_t* value) {
    return HRESULT_FROM_WIN32(RegSetValueExW(key, name, 0, REG_SZ,
        reinterpret_cast<const BYTE*>(value), static_cast<DWORD>((wcslen(value) + 1) * sizeof(wchar_t))));
}
HRESULT registerCom(const wchar_t* path) {
    HKEY key = nullptr;
    std::wstring subkey = std::wstring(ClassKey) + L"\\InprocServer32";
    auto result = RegCreateKeyExW(HKEY_LOCAL_MACHINE, subkey.c_str(), 0, nullptr, 0, KEY_WRITE, nullptr, &key, nullptr);
    if (result != ERROR_SUCCESS) return HRESULT_FROM_WIN32(result);
    auto status = writeString(key, nullptr, path);
    if (SUCCEEDED(status)) status = writeString(key, L"ThreadingModel", L"Apartment");
    RegCloseKey(key);
    return status;
}
HRESULT registerTsf(const wchar_t* path) {
    ComPtr<ITfInputProcessorProfileMgr> profiles;
    auto result = CoCreateInstance(CLSID_TF_InputProcessorProfiles, nullptr, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&profiles));
    if (FAILED(result)) return result;
    constexpr wchar_t name[] = L"Amu";
    result = profiles->RegisterProfile(ServiceId, LanguageId, ProfileId, name, ARRAYSIZE(name) - 1,
        path, static_cast<ULONG>(wcslen(path)), 0, nullptr, 0, TRUE, 0);
    if (FAILED(result)) return result;
    ComPtr<ITfCategoryMgr> categories;
    result = CoCreateInstance(CLSID_TF_CategoryMgr, nullptr, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&categories));
    if (FAILED(result)) return result;
    for (const auto& category : Categories) {
        result = categories->RegisterCategory(ServiceId, category, ServiceId);
        if (FAILED(result)) return result;
    }
    return S_OK;
}
HRESULT unregisterTsf() {
    ComPtr<ITfInputProcessorProfileMgr> profiles;
    auto result = CoCreateInstance(CLSID_TF_InputProcessorProfiles, nullptr, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&profiles));
    if (FAILED(result)) return result;
    auto status = profiles->UnregisterProfile(ServiceId, LanguageId, ProfileId, 0);
    ComPtr<ITfCategoryMgr> categories;
    result = CoCreateInstance(CLSID_TF_CategoryMgr, nullptr, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&categories));
    if (FAILED(result)) return result;
    for (const auto& category : Categories) categories->UnregisterCategory(ServiceId, category, ServiceId);
    return status;
}
}
}
extern "C" HRESULT __stdcall DllRegisterServer() {
    auto initialized = CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);
    if (FAILED(initialized) && initialized != RPC_E_CHANGED_MODE) return initialized;
    wchar_t path[32768]{};
    auto length = GetModuleFileNameW(amu::moduleHandle, path, ARRAYSIZE(path));
    auto result = (!length || length >= ARRAYSIZE(path)) ? E_FAIL : amu::registerCom(path);
    if (SUCCEEDED(result)) result = amu::registerTsf(path);
    if (SUCCEEDED(initialized)) CoUninitialize();
    return result;
}
extern "C" HRESULT __stdcall DllUnregisterServer() {
    auto initialized = CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);
    if (FAILED(initialized) && initialized != RPC_E_CHANGED_MODE) return initialized;
    auto result = amu::unregisterTsf();
    auto removed = RegDeleteTreeW(HKEY_LOCAL_MACHINE, amu::ClassKey);
    if (SUCCEEDED(result) && removed != ERROR_FILE_NOT_FOUND) result = HRESULT_FROM_WIN32(removed);
    if (SUCCEEDED(initialized)) CoUninitialize();
    return result;
}
