#pragma once
#include "context_session.h"
#include "key_input.h"

namespace amu {
class TextService final : public ITfTextInputProcessorEx, public ITfKeyEventSink,
    public ITfThreadMgrEventSink, public ITfThreadFocusSink, public ITfDisplayAttributeProvider, public Lifetime {
public:
    HRESULT STDMETHODCALLTYPE Activate(ITfThreadMgr* manager, TfClientId client) override;
    HRESULT STDMETHODCALLTYPE ActivateEx(ITfThreadMgr* manager, TfClientId client, DWORD flags) override;
    HRESULT STDMETHODCALLTYPE Deactivate() override;
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void** result) override;
    ULONG STDMETHODCALLTYPE AddRef() override { return retain(); }
    ULONG STDMETHODCALLTYPE Release() override { return release(); }
    HRESULT STDMETHODCALLTYPE OnTestKeyDown(ITfContext* context, WPARAM key, LPARAM, BOOL* eaten) override;
    HRESULT STDMETHODCALLTYPE OnKeyDown(ITfContext* context, WPARAM key, LPARAM, BOOL* eaten) override;
    HRESULT STDMETHODCALLTYPE OnTestKeyUp(ITfContext*, WPARAM, LPARAM, BOOL* eaten) override;
    HRESULT STDMETHODCALLTYPE OnKeyUp(ITfContext*, WPARAM, LPARAM, BOOL* eaten) override;
    HRESULT STDMETHODCALLTYPE OnPreservedKey(ITfContext*, REFGUID, BOOL* eaten) override;
    HRESULT STDMETHODCALLTYPE OnSetFocus(BOOL foreground) override;
    HRESULT STDMETHODCALLTYPE OnInitDocumentMgr(ITfDocumentMgr*) override { return S_OK; }
    HRESULT STDMETHODCALLTYPE OnUninitDocumentMgr(ITfDocumentMgr*) override { return S_OK; }
    HRESULT STDMETHODCALLTYPE OnSetFocus(ITfDocumentMgr*, ITfDocumentMgr*) override;
    HRESULT STDMETHODCALLTYPE OnPushContext(ITfContext*) override;
    HRESULT STDMETHODCALLTYPE OnPopContext(ITfContext*) override;
    HRESULT STDMETHODCALLTYPE OnSetThreadFocus() override { return S_OK; }
    HRESULT STDMETHODCALLTYPE OnKillThreadFocus() override;
    HRESULT STDMETHODCALLTYPE EnumDisplayAttributeInfo(IEnumTfDisplayAttributeInfo** result) override;
    HRESULT STDMETHODCALLTYPE GetDisplayAttributeInfo(REFGUID guid, ITfDisplayAttributeInfo** result) override;
private:
    bool accepts(ITfContext* context) const;
    HRESULT selectContext(ITfContext* context);
    void retireContext();
    ComPtr<ITfThreadMgr> manager_;
    ComPtr<ContextSession> session_;
    TfClientId client_ = TF_CLIENTID_NULL;
    TfGuidAtom attribute_ = TF_INVALID_GUIDATOM;
    DWORD managerSink_ = TF_INVALID_COOKIE;
    DWORD focusSink_ = TF_INVALID_COOKIE;
    bool advisedKeys_ = false;
    bool secure_ = false;
    uint32_t mode_ = 0;
};
}
