#include "text_store.h"
#include <algorithm>

namespace amu::test {
HRESULT TextStore::QueryInterface(REFIID iid, void** result) {
    if (!result) return E_POINTER;
    *result = nullptr;
    if (iid == IID_IUnknown || iid == IID_ITextStoreACP) *result = static_cast<ITextStoreACP*>(this);
    else if (iid == IID_ITfContextOwnerCompositionSink) *result = static_cast<ITfContextOwnerCompositionSink*>(this);
    else return E_NOINTERFACE;
    AddRef(); return S_OK;
}
HRESULT TextStore::AdviseSink(REFIID iid, IUnknown* sink, DWORD) {
    if (iid != IID_ITextStoreACPSink) return E_INVALIDARG;
    return sink->QueryInterface(IID_PPV_ARGS(&sink_));
}
HRESULT TextStore::UnadviseSink(IUnknown*) { sink_.Reset(); return S_OK; }
HRESULT TextStore::RequestLock(DWORD flags, HRESULT* result) {
    if (!result) return E_POINTER;
    if (denyLock || lock_) { *result = TS_E_SYNCHRONOUS; return S_OK; }
    if (!sink_) return E_UNEXPECTED;
    lock_ = flags;
    *result = sink_->OnLockGranted(flags);
    lock_ = 0;
    return S_OK;
}
HRESULT TextStore::GetStatus(TS_STATUS* status) { *status = {readOnly ? TS_SD_READONLY : 0, 0}; return S_OK; }
HRESULT TextStore::QueryInsert(LONG first, LONG last, ULONG, LONG* begin, LONG* finish) {
    if (first < 0 || last < first || last > static_cast<LONG>(text.size())) return TS_E_INVALIDPOS;
    *begin = first; *finish = last; return S_OK;
}
HRESULT TextStore::GetSelection(ULONG index, ULONG count, TS_SELECTION_ACP* selection, ULONG* fetched) {
    if (!(lock_ & TS_LF_READ)) return TS_E_NOLOCK;
    *fetched = 0;
    if (!count || (index != 0 && index != TS_DEFAULT_SELECTION)) return E_INVALIDARG;
    *selection = {start, end, {TS_AE_END, FALSE}}; *fetched = 1; return S_OK;
}
HRESULT TextStore::SetSelection(ULONG count, const TS_SELECTION_ACP* selection) {
    if (!(lock_ & TS_LF_READWRITE)) return TS_E_NOLOCK;
    if (count != 1) return E_INVALIDARG;
    start = selection->acpStart; end = selection->acpEnd; return S_OK;
}
HRESULT TextStore::GetText(LONG first, LONG last, WCHAR* plain, ULONG capacity, ULONG* copied,
    TS_RUNINFO* runs, ULONG runCapacity, ULONG* runCount, LONG* next) {
    if (!(lock_ & TS_LF_READ)) return TS_E_NOLOCK;
    if (last == -1) last = static_cast<LONG>(text.size());
    if (first < 0 || last < first || last > static_cast<LONG>(text.size())) return TS_E_INVALIDPOS;
    ULONG length = std::min(capacity, static_cast<ULONG>(last - first));
    if (length) std::copy_n(text.data() + first, length, plain);
    if (copied) *copied = length;
    if (runCount) *runCount = runCapacity && length ? 1 : 0;
    if (runCapacity && length) *runs = {length, TS_RT_PLAIN};
    if (next) *next = first + static_cast<LONG>(length);
    return S_OK;
}
HRESULT TextStore::SetText(DWORD, LONG first, LONG last, const WCHAR* value, ULONG length, TS_TEXTCHANGE* change) {
    if ((lock_ & TS_LF_READWRITE) != TS_LF_READWRITE) return TS_E_NOLOCK;
    if (readOnly) return TS_E_READONLY;
    if (first < 0 || last < first || last > static_cast<LONG>(text.size())) return TS_E_INVALIDPOS;
    text.replace(first, last - first, value ? value : L"", length);
    *change = {first, last, first + static_cast<LONG>(length)};
    start = end = change->acpNewEnd;
    return S_OK;
}
HRESULT TextStore::InsertTextAtSelection(DWORD flags, const WCHAR* value, ULONG length, LONG* first, LONG* last, TS_TEXTCHANGE* change) {
    if (first) *first = start;
    if (last) *last = end;
    if (flags & TS_IAS_QUERYONLY) return S_OK;
    return SetText(0, start, end, value, length, change);
}
HRESULT TextStore::FindNextAttrTransition(LONG, LONG halt, ULONG, const TS_ATTRID*, DWORD, LONG* next, BOOL* found, LONG* offset) {
    *next = halt; *found = FALSE; *offset = 0; return S_OK;
}
HRESULT TextStore::GetTextExt(TsViewCookie, LONG, LONG, RECT* rect, BOOL* clipped) {
    *rect = {0, 0, 100, 20}; *clipped = FALSE; return S_OK;
}
void TextStore::moveCaret(LONG position) { start = end = position; if (sink_) sink_->OnSelectionChange(); }
}
