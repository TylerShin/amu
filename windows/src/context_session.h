#pragma once
#include "composition.h"

namespace amu {
class ContextSession final : public ITfCompositionSink, public ITfTextEditSink, public Lifetime {
public:
    ContextSession(ITfContext* context, TfClientId client, TfGuidAtom attribute, uint32_t mode)
        : context_(context), client_(client), attribute_(attribute), core_(mode) {}
    HRESULT attach();
    void retire();
    bool belongsTo(ITfContext* context) const { return context_.Get() == context; }
    bool available() const { return enabled_ && core_.available(); }
    bool composing() const { return composition_.active(); }
    HRESULT process(CoreInput input, bool& consumed);
    void finishLater();
    HRESULT edit(TfEditCookie cookie, CoreInput input, bool& consumed);
    HRESULT finishEdit(TfEditCookie cookie, uint64_t revision);
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void** result) override;
    ULONG STDMETHODCALLTYPE AddRef() override { return retain(); }
    ULONG STDMETHODCALLTYPE Release() override { return release(); }
    HRESULT STDMETHODCALLTYPE OnCompositionTerminated(TfEditCookie, ITfComposition* composition) override;
    HRESULT STDMETHODCALLTYPE OnEndEdit(ITfContext*, TfEditCookie cookie, ITfEditRecord*) override;
private:
    ComPtr<ITfContext> context_;
    TfClientId client_;
    TfGuidAtom attribute_;
    CoreSession core_;
    Composition composition_;
    DWORD editSink_ = TF_INVALID_COOKIE;
    uint64_t revision_ = 0;
    bool applying_ = false;
    bool enabled_ = true;
    bool finishing_ = false;
};
}
