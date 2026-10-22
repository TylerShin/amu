#include "core_session.h"
#include "dependency_loader.h"

namespace amu {
namespace {
HRESULT decode(const uint8_t* bytes, uint32_t length, std::wstring& text);
}
CoreSession::CoreSession(uint32_t mode) : handle_(nullptr) {
    if (SUCCEEDED(prepareCoreLibrary())) handle_ = amu_session_new(mode);
}
HRESULT CoreSession::process(CoreInput input, CompositionUpdate& update) {
    if (!handle_) return E_UNEXPECTED;
    AmuOutput output{};
    if (amu_session_process(handle_, input.kind, input.value, &output) != 0) {
        amu_session_free(handle_);
        handle_ = nullptr;
        return E_FAIL;
    }
    update.consumed = output.consumed != 0;
    auto result = decode(output.commit, output.commit_len, update.commit);
    if (FAILED(result)) return result;
    return decode(output.preedit, output.preedit_len, update.preedit);
}
void CoreSession::reset() {
    if (handle_ && amu_session_reset(handle_) != 0) {
        amu_session_free(handle_);
        handle_ = nullptr;
    }
}
namespace {
HRESULT decode(const uint8_t* bytes, uint32_t length, std::wstring& text) {
    if (length > 1024) return E_UNEXPECTED;
    text.clear();
    if (!length) return S_OK;
    int size = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS,
        reinterpret_cast<const char*>(bytes), static_cast<int>(length), nullptr, 0);
    if (!size) return HRESULT_FROM_WIN32(GetLastError());
    text.resize(size);
    if (!MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS,
        reinterpret_cast<const char*>(bytes), static_cast<int>(length), text.data(), size)) {
        return HRESULT_FROM_WIN32(GetLastError());
    }
    return S_OK;
}
}

}
