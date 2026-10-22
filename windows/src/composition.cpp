#include "composition.h"
#include <vector>

namespace amu {
HRESULT Composition::apply(ITfContext* context, TfEditCookie cookie, ITfCompositionSink* sink,
    TfGuidAtom attribute, const CompositionUpdate& update) {
    if (!update.commit.empty()) {
        auto result = write(context, cookie, sink, attribute, update.commit);
        if (FAILED(result)) return result;
        result = finish(cookie);
        if (FAILED(result)) return result;
    }
    if (!update.preedit.empty()) return write(context, cookie, sink, attribute, update.preedit);
    if (active()) {
        auto result = write(context, cookie, sink, attribute, L"");
        if (FAILED(result)) return result;
        return finish(cookie);
    }
    return S_OK;
}
HRESULT Composition::finish(TfEditCookie cookie) {
    if (!composition_) return S_OK;
    // The document already contains the displayed text: never insert it a second time.
    ComPtr<ITfRange> range;
    auto result = composition_->GetRange(&range);
    if (FAILED(result)) return result;
    ComPtr<ITfContext> context;
    result = range->GetContext(&context);
    if (FAILED(result)) return result;
    ComPtr<ITfProperty> property;
    if (SUCCEEDED(context->GetProperty(GUID_PROP_ATTRIBUTE, &property))) property->Clear(cookie, range.Get());
    auto composition = composition_;
    ending_ = true;
    result = composition->EndComposition(cookie);
    ending_ = false;
    if (SUCCEEDED(result)) terminated();
    return result;
}
bool Composition::ownsCaretAndText(ITfContext* context, TfEditCookie cookie) const {
    if (!composition_) return true;
    TF_SELECTION selection{};
    ULONG fetched = 0;
    if (FAILED(context->GetSelection(cookie, TF_DEFAULT_SELECTION, 1, &selection, &fetched)) || fetched != 1) return false;
    ComPtr<ITfRange> selected;
    selected.Attach(selection.range);
    ComPtr<ITfRange> range;
    if (FAILED(composition_->GetRange(&range))) return false;
    LONG start = 1, end = 1;
    if (FAILED(selected->CompareStart(cookie, range.Get(), TF_ANCHOR_END, &start)) ||
        FAILED(selected->CompareEnd(cookie, range.Get(), TF_ANCHOR_END, &end)) || start != 0 || end != 0) return false;
    std::vector<wchar_t> text(displayed_.size() + 1);
    ULONG copied = 0;
    if (FAILED(range->GetText(cookie, 0, text.data(), static_cast<ULONG>(text.size()), &copied))) return false;
    return std::wstring(text.data(), copied) == displayed_;
}
HRESULT Composition::write(ITfContext* context, TfEditCookie cookie, ITfCompositionSink* sink,
    TfGuidAtom attribute, const std::wstring& text) {
    ComPtr<ITfRange> range;
    HRESULT result;
    if (composition_) {
        result = composition_->GetRange(&range);
    } else {
        ComPtr<ITfInsertAtSelection> insertion;
        result = context->QueryInterface(IID_PPV_ARGS(&insertion));
        if (FAILED(result)) return result;
        result = insertion->InsertTextAtSelection(cookie, TF_IAS_QUERYONLY, nullptr, 0, &range);
        if (FAILED(result)) return result;
        ComPtr<ITfContextComposition> compositions;
        result = context->QueryInterface(IID_PPV_ARGS(&compositions));
        if (FAILED(result)) return result;
        result = compositions->StartComposition(cookie, range.Get(), sink, &composition_);
        if (SUCCEEDED(result) && !composition_) return E_FAIL;
    }
    if (FAILED(result)) return result;
    if (displayed_ != text || text.empty()) {
        result = range->SetText(cookie, 0, text.data(), static_cast<LONG>(text.size()));
        if (FAILED(result)) return result;
        ++writes_;
        displayed_ = text;
    }
    ComPtr<ITfProperty> property;
    result = context->GetProperty(GUID_PROP_ATTRIBUTE, &property);
    if (FAILED(result)) return result;
    VARIANT value{};
    value.vt = VT_I4;
    value.lVal = static_cast<LONG>(attribute);
    result = property->SetValue(cookie, range.Get(), &value);
    if (FAILED(result)) return result;
    ComPtr<ITfRange> caret;
    result = range->Clone(&caret);
    if (FAILED(result)) return result;
    result = caret->Collapse(cookie, TF_ANCHOR_END);
    if (FAILED(result)) return result;
    TF_SELECTION selection{caret.Get(), {TF_AE_NONE, FALSE}};
    return context->SetSelection(cookie, 1, &selection);
}
}
