#pragma once
#include "core_session.h"

namespace amu {
// Owns only the TSF range. Rust owns interpretation and deletion rules.
class Composition {
public:
    bool active() const { return composition_ != nullptr; }
    bool ending() const { return ending_; }
    uint64_t writes() const { return writes_; }
    HRESULT apply(ITfContext* context, TfEditCookie cookie, ITfCompositionSink* sink,
        TfGuidAtom attribute, const CompositionUpdate& update);
    HRESULT finish(TfEditCookie cookie);
    bool matches(ITfComposition* value) const { return composition_.Get() == value; }
    void terminated() { composition_.Reset(); displayed_.clear(); }
    bool ownsCaretAndText(ITfContext* context, TfEditCookie cookie) const;
private:
    HRESULT write(ITfContext* context, TfEditCookie cookie, ITfCompositionSink* sink,
        TfGuidAtom attribute, const std::wstring& text);
    ComPtr<ITfComposition> composition_;
    std::wstring displayed_;
    uint64_t writes_ = 0;
    bool ending_ = false;
};
}
