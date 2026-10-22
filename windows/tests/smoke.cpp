#include "text_store.h"
#include "core_session.h"
#include "key_input.h"
#include <iostream>
#include <stdexcept>

using namespace amu;
using amu::test::TextStore;
namespace {
void require(bool condition, const char* message) { if (!condition) throw std::runtime_error(message); }
void check(HRESULT result, const char* message) {
    if (FAILED(result)) { std::cerr << message << " HRESULT=0x" << std::hex << result << std::dec << '\n'; throw std::runtime_error(message); }
}
void pump() {
    MSG message;
    while (PeekMessageW(&message, nullptr, 0, 0, PM_REMOVE)) { TranslateMessage(&message); DispatchMessageW(&message); }
}
struct Document {
    ComPtr<TextStore> store;
    ComPtr<ITfDocumentMgr> manager;
    ComPtr<ITfContext> context;
    Document(ITfThreadMgr* thread, TfClientId client) {
        store.Attach(new TextStore());
        check(thread->CreateDocumentMgr(&manager), "CreateDocumentMgr");
        TfEditCookie cookie;
        check(manager->CreateContext(client, 0, static_cast<ITextStoreACP*>(store.Get()), &context, &cookie), "CreateContext");
        check(manager->Push(context.Get()), "Push context");
    }
    ~Document() { manager->Pop(TF_POPF_ALL); }
};
bool keyDown(ITfKeyEventSink* keys, ITfContext* context, WPARAM key, bool shift = false, bool control = false) {
    BYTE state[256]{};
    if (shift) state[VK_SHIFT] = 0x80;
    if (control) state[VK_CONTROL] = 0x80;
    require(SetKeyboardState(state) != FALSE, "SetKeyboardState");
    BOOL tested = FALSE, eaten = FALSE;
    check(keys->OnTestKeyDown(context, key, 0, &tested), "OnTestKeyDown");
    if (tested) check(keys->OnKeyDown(context, key, 0, &eaten), "OnKeyDown");
    return eaten != FALSE;
}
void type(ITfKeyEventSink* keys, ITfContext* context, const char* text) {
    for (const char* character = text; *character; ++character) {
        bool upper = *character >= 'A' && *character <= 'Z';
        WPARAM key = upper ? *character : *character - 'a' + 'A';
        require(keyDown(keys, context, key, upper), "letter was not consumed");
    }
}
void testCoreAndKeys() {
    CoreSession core(0);
    CompositionUpdate update;
    for (char key : std::string("airh")) check(core.process({0, static_cast<uint32_t>(key)}, update), "core");
    require(update.preedit == L"ai고", "mixed core boundary");
    check(core.process({1, 0}, update), "delete");
    require(update.preedit == L"aiㄱ", "mixed deletion");
    auto shifted = translateKey('A', {true, false, false, false, false});
    require(shifted.core.value == 'A', "Shift A");
    require(translateKey('A', {false, false, false, false, true}).core.value == 'A', "Caps Lock");
    require(translateKey('A', {false, true, false, false, false}).action == KeyAction::finish, "Ctrl shortcut");
}
void testTsf(IClassFactory* factory) {
    ComPtr<ITfThreadMgrEx> thread;
    check(CoCreateInstance(CLSID_TF_ThreadMgr, nullptr, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&thread)), "TSF thread manager");
    TfClientId client;
    check(thread->ActivateEx(&client, TF_TMAE_NOACTIVATETIP), "Activate TSF");
    ComPtr<ITfClientId> clients;
    check(thread.As(&clients), "TSF client identifiers");
    TfClientId serviceClient;
    check(clients->GetClientId(ServiceId, &serviceClient), "text service client identifier");
    ComPtr<ITfTextInputProcessor> service;
    check(factory->CreateInstance(nullptr, IID_PPV_ARGS(&service)), "Create text service");
    check(service->Activate(thread.Get(), serviceClient), "Activate Amu");
    ComPtr<ITfKeyEventSink> keys;
    check(service.As(&keys), "Key sink");
    {
        Document document(thread.Get(), client);
        check(thread->SetFocus(document.manager.Get()), "Set focus");
        type(keys.Get(), document.context.Get(), "dkssud");
        require(document.store->text == L"안녕", "TSF Hangul composition");
        require(document.store->compositions == 1, "one live composition");
        require(keyDown(keys.Get(), document.context.Get(), VK_BACK), "Backspace consumed");
        require(document.store->text == L"안녀", "Hangul deletes one jamo");
        type(keys.Get(), document.context.Get(), "d");
        require(!keyDown(keys.Get(), document.context.Get(), VK_SPACE), "Space passes through");
        require(document.store->text == L"안녕", "boundary does not duplicate text");
        require(document.store->compositions == 0, "Space ends composition");
        require(!keyDown(keys.Get(), document.context.Get(), VK_RETURN), "Return passes through");
        require(document.store->text == L"안녕", "repeated finish is idempotent");
        document.store->denyLock = true;
        require(!keyDown(keys.Get(), document.context.Get(), 'A'), "denied edit forwards key");
        require(document.store->text == L"안녕", "denied edit does not change document");
        document.store->denyLock = false;
        type(keys.Get(), document.context.Get(), "AI");
        require(document.store->text == L"안녕AI", "uppercase survives denied edit");
        Document other(thread.Get(), client);
        check(thread->SetFocus(other.manager.Get()), "switch focus");
        pump();
        require(document.store->text == L"안녕AI", "focus preserves previous document");
        require(document.store->compositions == 0, "focus finishes previous composition");
        type(keys.Get(), other.context.Get(), "fallback");
        require(other.store->text == L"fallback", "English boundary");
        require(!keyDown(keys.Get(), other.context.Get(), 'C', false, true), "Ctrl+C passes through");
        require(other.store->text == L"fallback", "shortcut preserves text");
        require(other.store->compositions == 0, "shortcut finishes composition");
        type(keys.Get(), other.context.Get(), "fd");
        require(other.store->text == L"fallbackㄹㅇ", "initials use shared core");
        other.store->moveCaret(0);
        pump();
        type(keys.Get(), other.context.Get(), "AI");
        require(other.store->text == L"AIfallbackㄹㅇ", "moving caret preserves the previous composition");
        require(keyDown(keys.Get(), other.context.Get(), VK_F2, true, true), "Korean mode shortcut");
        type(keys.Get(), other.context.Get(), "rkqtk");
        require(other.store->text == L"AI갑사fallbackㄹㅇ", "fixed Korean mode at moved caret");
        require(keyDown(keys.Get(), other.context.Get(), VK_BACK), "Korean deletion consumed");
        require(other.store->text == L"AI갑ㅅfallbackㄹㅇ", "settled syllable deletion");
        check(thread->SetFocus(nullptr), "clear focus");
        pump();
    }
    {
        Document document(thread.Get(), client);
        check(thread->SetFocus(document.manager.Get()), "long input focus");
        require(keyDown(keys.Get(), document.context.Get(), VK_F3, true, true), "English mode shortcut");
        std::string letters(513, 'a');
        type(keys.Get(), document.context.Get(), letters.c_str());
        require(!keyDown(keys.Get(), document.context.Get(), VK_RETURN), "finish long input");
        require(document.store->text == std::wstring(513, L'a'), "bounded commits never duplicate or lose letters");
        check(thread->SetFocus(nullptr), "clear long input focus");
        pump();
    }
    check(service->Deactivate(), "Deactivate Amu");
    service.Reset(); keys.Reset();
    pump();
    check(thread->Deactivate(), "Deactivate TSF");
}
}
int wmain(int argc, wchar_t** argv) {
    try {
        HMODULE library = LoadLibraryExW(L"Amu.dll", nullptr, LOAD_LIBRARY_SEARCH_APPLICATION_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32);
        require(library != nullptr, "load Amu.dll");
        auto identity = reinterpret_cast<const char* (__cdecl*)()>(GetProcAddress(library, "AmuBuildJson"));
        require(identity != nullptr, "build identity export");
        if (argc == 2 && std::wstring(argv[1]) == L"--version") { std::cout << identity() << '\n'; FreeLibrary(library); return 0; }
        check(CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED), "COM apartment");
        auto getFactory = reinterpret_cast<HRESULT (__stdcall*)(REFCLSID, REFIID, void**)>(GetProcAddress(library, "DllGetClassObject"));
        require(getFactory != nullptr, "class factory export");
        ComPtr<IClassFactory> factory;
        if (argc == 2 && std::wstring(argv[1]) == L"--registered") {
            check(CoGetClassObject(ServiceId, CLSCTX_INPROC_SERVER, nullptr, IID_PPV_ARGS(&factory)), "registered COM class factory");
        } else {
            check(getFactory(ServiceId, IID_PPV_ARGS(&factory)), "class factory");
        }
        testCoreAndKeys();
        testTsf(factory.Get());
        factory.Reset();
        CoUninitialize();
        std::cout << "PASS: Windows C ABI, key translation and real TSF composition/edit/focus checks\n";
        FreeLibrary(library);
        return 0;
    } catch (const std::exception& error) { std::cerr << "FAIL: " << error.what() << '\n'; return 1; }
}
