#pragma once
#include "common.h"
#include "amu.h"

namespace amu {
struct CoreInput { uint32_t kind = 2; uint32_t value = 0; };
struct CompositionUpdate { bool consumed = false; std::wstring commit; std::wstring preedit; };

class CoreSession {
public:
    explicit CoreSession(uint32_t mode);
    ~CoreSession() { amu_session_free(handle_); }
    CoreSession(const CoreSession&) = delete;
    CoreSession& operator=(const CoreSession&) = delete;
    bool available() const { return handle_ != nullptr; }
    HRESULT process(CoreInput input, CompositionUpdate& update);
    void reset();
private:
    AmuSession* handle_;
};
}
