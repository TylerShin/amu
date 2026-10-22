#include "text_service.h"

namespace amu {
namespace {
class AttributeInfo final : public ITfDisplayAttributeInfo, public Lifetime {
public:
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void** result) override {
        if (!result) return E_POINTER;
        *result = nullptr;
        if (iid != IID_IUnknown && iid != IID_ITfDisplayAttributeInfo) return E_NOINTERFACE;
        *result = static_cast<ITfDisplayAttributeInfo*>(this); AddRef(); return S_OK;
    }
    ULONG STDMETHODCALLTYPE AddRef() override { return retain(); }
    ULONG STDMETHODCALLTYPE Release() override { return release(); }
    HRESULT STDMETHODCALLTYPE GetGUID(GUID* guid) override { if (!guid) return E_POINTER; *guid = UnderlineId; return S_OK; }
    HRESULT STDMETHODCALLTYPE GetDescription(BSTR* text) override {
        if (!text) return E_POINTER;
        *text = SysAllocString(L"Amu composition"); return *text ? S_OK : E_OUTOFMEMORY;
    }
    HRESULT STDMETHODCALLTYPE GetAttributeInfo(TF_DISPLAYATTRIBUTE* value) override {
        if (!value) return E_POINTER;
        *value = {};
        value->lsStyle = TF_LS_SOLID;
        value->bAttr = TF_ATTR_INPUT;
        // Host owns font and foreground/background colours.
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE SetAttributeInfo(const TF_DISPLAYATTRIBUTE*) override { return E_NOTIMPL; }
    HRESULT STDMETHODCALLTYPE Reset() override { return S_OK; }
};
class AttributeEnumerator final : public IEnumTfDisplayAttributeInfo, public Lifetime {
public:
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void** result) override {
        if (!result) return E_POINTER;
        *result = nullptr;
        if (iid != IID_IUnknown && iid != IID_IEnumTfDisplayAttributeInfo) return E_NOINTERFACE;
        *result = static_cast<IEnumTfDisplayAttributeInfo*>(this); AddRef(); return S_OK;
    }
    ULONG STDMETHODCALLTYPE AddRef() override { return retain(); }
    ULONG STDMETHODCALLTYPE Release() override { return release(); }
    HRESULT STDMETHODCALLTYPE Clone(IEnumTfDisplayAttributeInfo** result) override {
        if (!result) return E_POINTER;
        auto copy = new (std::nothrow) AttributeEnumerator();
        *result = copy;
        if (!copy) return E_OUTOFMEMORY;
        copy->done_ = done_; return S_OK;
    }
    HRESULT STDMETHODCALLTYPE Next(ULONG count, ITfDisplayAttributeInfo** result, ULONG* fetched) override {
        if (!result || (!fetched && count != 1)) return E_POINTER;
        if (fetched) *fetched = 0;
        if (!count) return S_OK;
        *result = nullptr;
        if (done_) return S_FALSE;
        *result = new (std::nothrow) AttributeInfo();
        if (!*result) return E_OUTOFMEMORY;
        done_ = true;
        if (fetched) *fetched = 1;
        return count == 1 ? S_OK : S_FALSE;
    }
    HRESULT STDMETHODCALLTYPE Reset() override { done_ = false; return S_OK; }
    HRESULT STDMETHODCALLTYPE Skip(ULONG count) override {
        if (!count) return S_OK;
        bool hadItem = !done_; done_ = true;
        return hadItem && count == 1 ? S_OK : S_FALSE;
    }
private:
    bool done_ = false;
};
}
HRESULT TextService::EnumDisplayAttributeInfo(IEnumTfDisplayAttributeInfo** result) {
    if (!result) return E_POINTER;
    *result = new (std::nothrow) AttributeEnumerator();
    return *result ? S_OK : E_OUTOFMEMORY;
}
HRESULT TextService::GetDisplayAttributeInfo(REFGUID guid, ITfDisplayAttributeInfo** result) {
    if (!result) return E_POINTER;
    *result = nullptr;
    if (guid != UnderlineId) return E_INVALIDARG;
    *result = new (std::nothrow) AttributeInfo();
    return *result ? S_OK : E_OUTOFMEMORY;
}
}
