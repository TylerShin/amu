#pragma once
#include "common.h"
#include <textstor.h>

namespace amu::test {
// A small real TSF document owner. This is not a mock of ITfContext or ITfRange.
class TextStore final : public ITextStoreACP, public ITfContextOwnerCompositionSink, public Lifetime {
public:
    TextStore() : window_(CreateWindowExW(0, L"STATIC", L"Amu TSF test", WS_OVERLAPPED, 0, 0, 800, 600, nullptr, nullptr, GetModuleHandleW(nullptr), nullptr)) {}
    ~TextStore() override { if (window_) DestroyWindow(window_); }
    std::wstring text;
    LONG start = 0;
    LONG end = 0;
    bool denyLock = false;
    bool readOnly = false;
    unsigned compositions = 0;
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID, void**) override;
    ULONG STDMETHODCALLTYPE AddRef() override { return retain(); }
    ULONG STDMETHODCALLTYPE Release() override { return release(); }
    HRESULT STDMETHODCALLTYPE AdviseSink(REFIID, IUnknown*, DWORD) override;
    HRESULT STDMETHODCALLTYPE UnadviseSink(IUnknown*) override;
    HRESULT STDMETHODCALLTYPE RequestLock(DWORD, HRESULT*) override;
    HRESULT STDMETHODCALLTYPE GetStatus(TS_STATUS*) override;
    HRESULT STDMETHODCALLTYPE QueryInsert(LONG, LONG, ULONG, LONG*, LONG*) override;
    HRESULT STDMETHODCALLTYPE GetSelection(ULONG, ULONG, TS_SELECTION_ACP*, ULONG*) override;
    HRESULT STDMETHODCALLTYPE SetSelection(ULONG, const TS_SELECTION_ACP*) override;
    HRESULT STDMETHODCALLTYPE GetText(LONG, LONG, WCHAR*, ULONG, ULONG*, TS_RUNINFO*, ULONG, ULONG*, LONG*) override;
    HRESULT STDMETHODCALLTYPE SetText(DWORD, LONG, LONG, const WCHAR*, ULONG, TS_TEXTCHANGE*) override;
    HRESULT STDMETHODCALLTYPE GetFormattedText(LONG, LONG, IDataObject**) override { return E_NOTIMPL; }
    HRESULT STDMETHODCALLTYPE GetEmbedded(LONG, REFGUID, REFIID, IUnknown**) override { return E_NOTIMPL; }
    HRESULT STDMETHODCALLTYPE QueryInsertEmbedded(const GUID*, const FORMATETC*, BOOL* result) override { *result = FALSE; return S_OK; }
    HRESULT STDMETHODCALLTYPE InsertEmbedded(DWORD, LONG, LONG, IDataObject*, TS_TEXTCHANGE*) override { return E_NOTIMPL; }
    HRESULT STDMETHODCALLTYPE InsertTextAtSelection(DWORD, const WCHAR*, ULONG, LONG*, LONG*, TS_TEXTCHANGE*) override;
    HRESULT STDMETHODCALLTYPE InsertEmbeddedAtSelection(DWORD, IDataObject*, LONG*, LONG*, TS_TEXTCHANGE*) override { return E_NOTIMPL; }
    HRESULT STDMETHODCALLTYPE RequestSupportedAttrs(DWORD, ULONG, const TS_ATTRID*) override { return S_OK; }
    HRESULT STDMETHODCALLTYPE RequestAttrsAtPosition(LONG, ULONG, const TS_ATTRID*, DWORD) override { return S_OK; }
    HRESULT STDMETHODCALLTYPE RequestAttrsTransitioningAtPosition(LONG, ULONG, const TS_ATTRID*, DWORD) override { return S_OK; }
    HRESULT STDMETHODCALLTYPE FindNextAttrTransition(LONG, LONG, ULONG, const TS_ATTRID*, DWORD, LONG*, BOOL*, LONG*) override;
    HRESULT STDMETHODCALLTYPE RetrieveRequestedAttrs(ULONG, TS_ATTRVAL*, ULONG* fetched) override { *fetched = 0; return S_OK; }
    HRESULT STDMETHODCALLTYPE GetEndACP(LONG* result) override { *result = static_cast<LONG>(text.size()); return S_OK; }
    HRESULT STDMETHODCALLTYPE GetActiveView(TsViewCookie* result) override { *result = 0; return S_OK; }
    HRESULT STDMETHODCALLTYPE GetACPFromPoint(TsViewCookie, const POINT*, DWORD, LONG*) override { return TS_E_NOLAYOUT; }
    HRESULT STDMETHODCALLTYPE GetTextExt(TsViewCookie, LONG, LONG, RECT*, BOOL*) override;
    HRESULT STDMETHODCALLTYPE GetScreenExt(TsViewCookie, RECT* rect) override { *rect = {0, 0, 800, 600}; return S_OK; }
    HRESULT STDMETHODCALLTYPE GetWnd(TsViewCookie, HWND* window) override { *window = window_; return window_ ? S_OK : E_FAIL; }
    HRESULT STDMETHODCALLTYPE OnStartComposition(ITfCompositionView*, BOOL* accepted) override { ++compositions; *accepted = TRUE; return S_OK; }
    HRESULT STDMETHODCALLTYPE OnUpdateComposition(ITfCompositionView*, ITfRange*) override { return S_OK; }
    HRESULT STDMETHODCALLTYPE OnEndComposition(ITfCompositionView*) override { --compositions; return S_OK; }
    void moveCaret(LONG position);
    void insertFromHost(const std::wstring& value);
private:
    ComPtr<ITextStoreACPSink> sink_;
    DWORD lock_ = 0;
    DWORD pendingLock_ = 0;
    HWND window_ = nullptr;
};
}
