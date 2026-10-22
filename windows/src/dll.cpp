#include "text_service.h"
#include "build_identity.h"

namespace amu {
class ClassFactory final : public IClassFactory, public Lifetime {
public:
    HRESULT STDMETHODCALLTYPE CreateInstance(IUnknown* outer, REFIID iid, void** result) override {
        if (!result) return E_POINTER;
        *result = nullptr;
        if (outer) return CLASS_E_NOAGGREGATION;
        auto service = new (std::nothrow) TextService();
        if (!service) return E_OUTOFMEMORY;
        auto status = service->QueryInterface(iid, result);
        service->Release(); return status;
    }
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void** result) override {
        if (!result) return E_POINTER;
        *result = nullptr;
        if (iid != IID_IUnknown && iid != IID_IClassFactory) return E_NOINTERFACE;
        *result = static_cast<IClassFactory*>(this); AddRef(); return S_OK;
    }
    ULONG STDMETHODCALLTYPE AddRef() override { return retain(); }
    ULONG STDMETHODCALLTYPE Release() override { return release(); }
    HRESULT STDMETHODCALLTYPE LockServer(BOOL lock) override { if (lock) ++moduleObjects; else --moduleObjects; return S_OK; }
};
}
BOOL WINAPI DllMain(HINSTANCE module, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) { amu::moduleHandle = module; DisableThreadLibraryCalls(module); }
    return TRUE;
}
extern "C" HRESULT __stdcall DllGetClassObject(REFCLSID clsid, REFIID iid, void** result) {
    if (!result) return E_POINTER;
    *result = nullptr;
    if (clsid != amu::ServiceId) return CLASS_E_CLASSNOTAVAILABLE;
    auto factory = new (std::nothrow) amu::ClassFactory();
    if (!factory) return E_OUTOFMEMORY;
    auto status = factory->QueryInterface(iid, result);
    factory->Release(); return status;
}
extern "C" HRESULT __stdcall DllCanUnloadNow() { return amu::moduleObjects == 0 ? S_OK : S_FALSE; }
extern "C" const char* __cdecl AmuBuildJson() { return AMU_BUILD_JSON; }
