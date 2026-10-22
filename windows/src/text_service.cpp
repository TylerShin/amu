#include "text_service.h"
#include <cstdio>

namespace amu {
HRESULT TextService::Activate(ITfThreadMgr* manager, TfClientId client) { return ActivateEx(manager, client, 0); }
HRESULT TextService::ActivateEx(ITfThreadMgr* manager, TfClientId client, DWORD flags) {
    if (!manager || manager_) return E_INVALIDARG;
    manager_ = manager;
    client_ = client;
    secure_ = (flags & TF_TMAE_SECUREMODE) != 0;
    const char* stage = "create category manager";
    ComPtr<ITfCategoryMgr> categories;
    auto result = CoCreateInstance(CLSID_TF_CategoryMgr, nullptr, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&categories));
    if (SUCCEEDED(result)) { stage = "register display GUID"; result = categories->RegisterGUID(UnderlineId, &attribute_); }
    ComPtr<ITfSource> source;
    if (SUCCEEDED(result)) { stage = "get thread source"; result = manager_.As(&source); }
    if (SUCCEEDED(result)) { stage = "advise thread manager sink"; result = source->AdviseSink(IID_ITfThreadMgrEventSink, static_cast<ITfThreadMgrEventSink*>(this), &managerSink_); }
    if (SUCCEEDED(result)) { stage = "advise thread focus sink"; result = source->AdviseSink(IID_ITfThreadFocusSink, static_cast<ITfThreadFocusSink*>(this), &focusSink_); }
    ComPtr<ITfKeystrokeMgr> keys;
    if (SUCCEEDED(result)) { stage = "get keystroke manager"; result = manager_.As(&keys); }
    if (SUCCEEDED(result)) {
        stage = "advise key event sink";
        result = keys->AdviseKeyEventSink(client_, this, TRUE);
        advisedKeys_ = SUCCEEDED(result);
    }
    if (FAILED(result)) {
        std::fprintf(stderr, "Amu activation failed: %s (0x%08lx)\n", stage, static_cast<unsigned long>(result));
        Deactivate();
    }
    return result;
}
HRESULT TextService::Deactivate() {
    retireContext();
    if (manager_) {
        ComPtr<ITfKeystrokeMgr> keys;
        if (advisedKeys_ && SUCCEEDED(manager_.As(&keys))) keys->UnadviseKeyEventSink(client_);
        ComPtr<ITfSource> source;
        if (SUCCEEDED(manager_.As(&source))) {
            if (managerSink_ != TF_INVALID_COOKIE) source->UnadviseSink(managerSink_);
            if (focusSink_ != TF_INVALID_COOKIE) source->UnadviseSink(focusSink_);
        }
    }
    manager_.Reset();
    advisedKeys_ = false;
    managerSink_ = focusSink_ = TF_INVALID_COOKIE;
    client_ = TF_CLIENTID_NULL;
    return S_OK;
}
HRESULT TextService::OnTestKeyDown(ITfContext* context, WPARAM key, LPARAM, BOOL* eaten) {
    if (!eaten) return E_POINTER;
    *eaten = FALSE;
    if (!accepts(context)) { retireContext(); return S_OK; }
    const auto input = translateKey(key, keyboardModifiers());
    if (input.action == KeyAction::ignore) return S_OK;
    if (input.action == KeyAction::mode) { *eaten = TRUE; return S_OK; }
    if (input.action == KeyAction::input) {
        *eaten = key != VK_BACK || (session_ && session_->belongsTo(context) && session_->composing());
        if (!*eaten && session_) session_->finishLater();
        return S_OK;
    }
    // TSF may never call OnKeyDown after FALSE. Finish before forwarding the
    // boundary here, so Enter/Tab/shortcuts reach the host exactly once and in order.
    if (session_ && session_->belongsTo(context) && session_->available()) {
        bool consumed = false;
        session_->process(input.core, consumed);
    }
    return S_OK;
}
HRESULT TextService::OnKeyDown(ITfContext* context, WPARAM key, LPARAM, BOOL* eaten) {
    if (!eaten) return E_POINTER;
    *eaten = FALSE;
    if (!accepts(context)) { retireContext(); return S_OK; }
    try {
        auto input = translateKey(key, keyboardModifiers());
        if (input.action == KeyAction::ignore) return S_OK;
        auto result = selectContext(context);
        if (FAILED(result)) return S_OK;
        bool consumed = false;
        result = session_->process(input.core, consumed);
        *eaten = consumed;
        if (input.action == KeyAction::mode && SUCCEEDED(result)) { mode_ = input.core.value; *eaten = TRUE; }
        return S_OK;
    } catch (...) { retireContext(); return S_OK; }
}
HRESULT TextService::OnTestKeyUp(ITfContext*, WPARAM, LPARAM, BOOL* eaten) {
    if (!eaten) return E_POINTER; *eaten = FALSE; return S_OK;
}
HRESULT TextService::OnKeyUp(ITfContext*, WPARAM, LPARAM, BOOL* eaten) {
    if (!eaten) return E_POINTER; *eaten = FALSE; return S_OK;
}
HRESULT TextService::OnPreservedKey(ITfContext*, REFGUID, BOOL* eaten) {
    if (!eaten) return E_POINTER; *eaten = FALSE; return S_OK;
}
HRESULT TextService::OnSetFocus(BOOL foreground) { if (!foreground) retireContext(); return S_OK; }
HRESULT TextService::OnSetFocus(ITfDocumentMgr*, ITfDocumentMgr*) { retireContext(); return S_OK; }
HRESULT TextService::OnPushContext(ITfContext*) { retireContext(); return S_OK; }
HRESULT TextService::OnPopContext(ITfContext*) { retireContext(); return S_OK; }
HRESULT TextService::OnKillThreadFocus() { retireContext(); return S_OK; }
HRESULT TextService::QueryInterface(REFIID iid, void** result) {
    if (!result) return E_POINTER;
    *result = nullptr;
    if (iid == IID_IUnknown || iid == IID_ITfTextInputProcessor || iid == IID_ITfTextInputProcessorEx) *result = static_cast<ITfTextInputProcessorEx*>(this);
    else if (iid == IID_ITfKeyEventSink) *result = static_cast<ITfKeyEventSink*>(this);
    else if (iid == IID_ITfThreadMgrEventSink) *result = static_cast<ITfThreadMgrEventSink*>(this);
    else if (iid == IID_ITfThreadFocusSink) *result = static_cast<ITfThreadFocusSink*>(this);
    else if (iid == IID_ITfDisplayAttributeProvider) *result = static_cast<ITfDisplayAttributeProvider*>(this);
    else return E_NOINTERFACE;
    AddRef(); return S_OK;
}
bool TextService::accepts(ITfContext* context) const {
    if (!context || !manager_ || secure_) return false;
    TF_STATUS status{};
    if (FAILED(context->GetStatus(&status)) || (status.dwDynamicFlags & TS_SD_READONLY)) return false;
    IUnknown* scopes[] = {context, manager_.Get()};
    for (auto scope : scopes) {
        ComPtr<ITfCompartmentMgr> compartments;
        if (FAILED(scope->QueryInterface(IID_PPV_ARGS(&compartments)))) return false;
        for (const auto& guid : {GUID_COMPARTMENT_KEYBOARD_DISABLED, GUID_COMPARTMENT_EMPTYCONTEXT}) {
            ComPtr<ITfCompartment> compartment;
            VARIANT value{};
            if (SUCCEEDED(compartments->GetCompartment(guid, &compartment)) && SUCCEEDED(compartment->GetValue(&value))) {
                bool disabled = value.vt == VT_I4 && value.lVal != 0;
                VariantClear(&value);
                if (disabled) return false;
            }
        }
    }
    return true;
}
HRESULT TextService::selectContext(ITfContext* context) {
    if (session_ && session_->belongsTo(context)) return session_->available() ? S_OK : E_FAIL;
    retireContext();
    session_.Attach(new (std::nothrow) ContextSession(context, client_, attribute_, mode_));
    if (!session_) return E_OUTOFMEMORY;
    auto result = session_->attach();
    if (FAILED(result)) session_.Reset();
    return result;
}
void TextService::retireContext() {
    if (!session_) return;
    auto previous = session_;
    session_.Reset();
    previous->retire();
}
}
