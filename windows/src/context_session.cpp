#include "context_session.h"

namespace amu {
namespace {
class InputEdit final : public ITfEditSession, public Lifetime {
public:
    InputEdit(ContextSession* session, CoreInput input) : session_(session), input_(input) {}
    InputEdit(ContextSession* session, uint64_t revision) : session_(session), finish_(true), revision_(revision) {}
    HRESULT STDMETHODCALLTYPE DoEditSession(TfEditCookie cookie) override {
        try {
            return finish_ ? session_->finishEdit(cookie, revision_) : session_->edit(cookie, input_, consumed);
        } catch (...) { return E_FAIL; }
    }
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void** result) override {
        if (!result) return E_POINTER;
        *result = nullptr;
        if (iid != IID_IUnknown && iid != IID_ITfEditSession) return E_NOINTERFACE;
        *result = static_cast<ITfEditSession*>(this); AddRef(); return S_OK;
    }
    ULONG STDMETHODCALLTYPE AddRef() override { return retain(); }
    ULONG STDMETHODCALLTYPE Release() override { return release(); }
    bool consumed = false;
private:
    ComPtr<ContextSession> session_;
    CoreInput input_{};
    bool finish_ = false;
    uint64_t revision_ = 0;
};
}
HRESULT ContextSession::attach() {
    if (!core_.available()) return E_OUTOFMEMORY;
    ComPtr<ITfSource> source;
    auto result = context_.As(&source);
    if (FAILED(result)) return result;
    return source->AdviseSink(IID_ITfTextEditSink, static_cast<ITfTextEditSink*>(this), &editSink_);
}
void ContextSession::retire() {
    ComPtr<ITfSource> source;
    if (editSink_ != TF_INVALID_COOKIE && SUCCEEDED(context_.As(&source))) {
        source->UnadviseSink(editSink_);
        editSink_ = TF_INVALID_COOKIE;
    }
    enabled_ = false;
    finishLater();
}
HRESULT ContextSession::process(CoreInput input, bool& consumed) {
    consumed = false;
    ComPtr<InputEdit> edit;
    edit.Attach(new (std::nothrow) InputEdit(this, input));
    if (!edit) return E_OUTOFMEMORY;
    HRESULT sessionResult = E_FAIL;
    auto result = context_->RequestEditSession(client_, edit.Get(), TF_ES_SYNC | TF_ES_READWRITE, &sessionResult);
    consumed = edit->consumed;
    // A refused lock must not advance Rust or swallow the original key.
    return FAILED(result) ? result : sessionResult;
}
void ContextSession::finishLater() {
    core_.reset();
    if (!composition_.active()) return;
    ComPtr<InputEdit> edit;
    edit.Attach(new (std::nothrow) InputEdit(this, revision_));
    if (!edit) return;
    HRESULT result = E_FAIL;
    context_->RequestEditSession(client_, edit.Get(), TF_ES_ASYNCDONTCARE | TF_ES_READWRITE, &result);
}
HRESULT ContextSession::edit(TfEditCookie cookie, CoreInput input, bool& consumed) {
    if (applying_) return E_UNEXPECTED;
    struct Applying { bool& value; Applying(bool& v) : value(v) { value = true; } ~Applying() { value = false; } } applying(applying_);
    ++revision_;
    if (!composition_.ownsCaretAndText(context_.Get(), cookie)) {
        auto result = composition_.finish(cookie);
        core_.reset();
        if (FAILED(result)) return result;
    }
    CompositionUpdate update;
    auto result = core_.process(input, update);
    if (FAILED(result)) {
        enabled_ = false;
        composition_.finish(cookie);
        return result;
    }
    auto writesBefore = composition_.writes();
    result = composition_.apply(context_.Get(), cookie, this, attribute_, update);
    if (FAILED(result)) {
        consumed = update.consumed && composition_.writes() != writesBefore;
        // Stop editing this context after a host failure; retain the host's current text.
        enabled_ = false;
        core_.reset();
        composition_.finish(cookie);
        return result;
    }
    consumed = update.consumed;
    return S_OK;
}
HRESULT ContextSession::finishEdit(TfEditCookie cookie, uint64_t revision) {
    if (revision != revision_) return S_OK;
    applying_ = true;
    auto result = composition_.finish(cookie);
    applying_ = false;
    return result;
}
HRESULT ContextSession::QueryInterface(REFIID iid, void** result) {
    if (!result) return E_POINTER;
    *result = nullptr;
    if (iid == IID_IUnknown || iid == IID_ITfCompositionSink) *result = static_cast<ITfCompositionSink*>(this);
    else if (iid == IID_ITfTextEditSink) *result = static_cast<ITfTextEditSink*>(this);
    else return E_NOINTERFACE;
    AddRef(); return S_OK;
}
HRESULT ContextSession::OnCompositionTerminated(TfEditCookie, ITfComposition* composition) {
    if (composition_.matches(composition)) {
        bool expected = composition_.ending();
        composition_.terminated();
        if (!expected) core_.reset();
        ++revision_;
    }
    return S_OK;
}
HRESULT ContextSession::OnEndEdit(ITfContext*, TfEditCookie cookie, ITfEditRecord*) {
    if (!applying_ && composition_.active() && !composition_.ownsCaretAndText(context_.Get(), cookie)) finishLater();
    return S_OK;
}
}
