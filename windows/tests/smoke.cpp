#include "text_store.h"
#include "core_session.h"
#include "context_session.h"
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
    ComPtr<ITfThreadMgr> owner;
    Document(ITfThreadMgr* thread, TfClientId client) : owner(thread) {
        store.Attach(new TextStore());
        check(thread->CreateDocumentMgr(&manager), "CreateDocumentMgr");
        TfEditCookie cookie;
        check(manager->CreateContext(client, 0, static_cast<ITextStoreACP*>(store.Get()), &context, &cookie), "CreateContext");
        check(manager->Push(context.Get()), "Push context");
        ComPtr<ITfDocumentMgr> previous;
        check(thread->AssociateFocus(store->window(), manager.Get(), &previous), "associate document window");
    }
    ~Document() {
        ComPtr<ITfDocumentMgr> previous;
        owner->AssociateFocus(store->window(), nullptr, &previous);
        manager->Pop(TF_POPF_ALL);
    }
};
bool keyDown(ITfKeystrokeMgr* keys, ITfContext*, WPARAM key, bool shift = false, bool control = false) {
    BYTE state[256]{};
    if (shift) state[VK_SHIFT] = 0x80;
    if (control) state[VK_CONTROL] = 0x80;
    require(SetKeyboardState(state) != FALSE, "SetKeyboardState");
    BOOL tested = FALSE, eaten = FALSE;
    check(keys->TestKeyDown(key, 0, &tested), "OnTestKeyDown");
    if (tested) check(keys->KeyDown(key, 0, &eaten), "OnKeyDown");
    if (!tested && key >= 'A' && key <= 'Z' && !control) {
        CLSID foreground{};
        auto result = keys->GetForeground(&foreground);
        std::cerr << "TEST: key route did not select Amu; foreground match=" << (foreground == ServiceId)
            << " result=0x" << std::hex << result << std::dec << '\n';
    }
    return eaten != FALSE;
}
void type(ITfKeystrokeMgr* keys, ITfContext* context, const char* text) {
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
void testRegisteredTsf() {
    std::cout << "TEST: installed profile and focused TSF document" << std::endl;
    ComPtr<ITfThreadMgrEx> thread;
    check(CoCreateInstance(CLSID_TF_ThreadMgr, nullptr, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&thread)), "TSF thread manager");
    TfClientId client;
    check(thread->ActivateEx(&client, 0), "Activate TSF");
    ComPtr<ITfInputProcessorProfiles> languages;
    check(CoCreateInstance(CLSID_TF_InputProcessorProfiles, nullptr, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&languages)), "language profiles");
    check(languages->ChangeCurrentLanguage(LanguageId), "test process Korean input language");
    ComPtr<ITfInputProcessorProfileMgr> profiles;
    check(languages.As(&profiles), "profile manager");
    auto activated = profiles->ActivateProfile(TF_PROFILETYPE_INPUTPROCESSOR, LanguageId, ServiceId, ProfileId, nullptr, TF_IPPMF_FORPROCESS);
    check(activated, "activate installed Amu profile");
    require(activated == S_OK, "installed Amu profile must be enabled");
    pump();
    TF_INPUTPROCESSORPROFILE active{};
    check(profiles->GetActiveProfile(GUID_TFCAT_TIP_KEYBOARD, &active), "active keyboard profile");
    require(active.clsid == ServiceId, "Windows must activate the installed Amu service");
    ComPtr<ITfKeystrokeMgr> keys;
    check(thread.As(&keys), "TSF keystroke manager");
    {
        Document document(thread.Get(), client);
        document.store->focusForTyping();
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
        other.store->focusForTyping();
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
        document.store->focusForTyping();
        check(thread->SetFocus(document.manager.Get()), "long input focus");
        require(keyDown(keys.Get(), document.context.Get(), VK_F3, true, true), "English mode shortcut");
        std::string letters(513, 'a');
        type(keys.Get(), document.context.Get(), letters.c_str());
        require(!keyDown(keys.Get(), document.context.Get(), VK_RETURN), "finish long input");
        require(document.store->text == std::wstring(513, L'a'), "bounded commits never duplicate or lose letters");
        check(thread->SetFocus(nullptr), "clear long input focus");
        pump();
    }
    {
        Document document(thread.Get(), client);
        document.store->focusForTyping();
        check(thread->SetFocus(document.manager.Get()), "mixed sentence focus");
        require(keyDown(keys.Get(), document.context.Get(), VK_F1, true, true), "auto mode shortcut");
        type(keys.Get(), document.context.Get(), "tkatjdsdi");
        require(!keyDown(keys.Get(), document.context.Get(), VK_SPACE), "host inserts the space");
        document.store->insertFromHost(L" ");
        type(keys.Get(), document.context.Get(), "tkrhtlvdj");
        require(!keyDown(keys.Get(), document.context.Get(), VK_RETURN), "finish mixed sentence");
        require(document.store->text == L"삼성sdi 사고싶어", "mixed sentence preserves one host space");
        ComPtr<ITfCompartmentMgr> compartments;
        check(document.context.As(&compartments), "context compartments");
        ComPtr<ITfCompartment> disabled;
        check(compartments->GetCompartment(GUID_COMPARTMENT_KEYBOARD_DISABLED, &disabled), "keyboard disabled compartment");
        VARIANT value{}; value.vt = VT_I4; value.lVal = 1;
        check(disabled->SetValue(client, &value), "disable composition");
        require(!keyDown(keys.Get(), document.context.Get(), 'A'), "disabled input context passes through");
        require(document.store->text == L"삼성sdi 사고싶어", "disabled context is unchanged");
        check(thread->SetFocus(nullptr), "clear sentence focus");
        pump();
    }
    check(profiles->DeactivateProfile(TF_PROFILETYPE_INPUTPROCESSOR, LanguageId, ServiceId, ProfileId, nullptr, TF_IPPMF_FORPROCESS), "deactivate test process Amu profile");
    keys.Reset();
    pump();
    check(thread->Deactivate(), "Deactivate TSF");
    std::cout << "PASS: installed TSF profile, keyboard routing, editing, focus and boundaries\n";
}
void testCompositionContext() {
    std::cout << "TEST: real TSF composition context" << std::endl;
    ComPtr<ITfThreadMgrEx> thread;
    check(CoCreateInstance(CLSID_TF_ThreadMgr, nullptr, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&thread)), "composition thread");
    TfClientId client;
    check(thread->ActivateEx(&client, TF_TMAE_NOACTIVATETIP), "composition thread activation");
    ComPtr<ITfCategoryMgr> categories;
    check(CoCreateInstance(CLSID_TF_CategoryMgr, nullptr, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&categories)), "composition category manager");
    TfGuidAtom attribute;
    check(categories->RegisterGUID(UnderlineId, &attribute), "composition attribute");
    {
        Document document(thread.Get(), client);
        check(thread->SetFocus(document.manager.Get()), "composition focus");
        document.store->insertFromHost(L"🙂 ");
        ComPtr<ContextSession> session;
        session.Attach(new ContextSession(document.context.Get(), client, attribute, 1));
        check(session->attach(), "composition sink");
        std::cout << "TEST: composition edit lock" << std::endl;
        bool consumed = false;
        for (char key : std::string("rkqtk")) check(session->process({0, static_cast<uint32_t>(key)}, consumed), "composition input");
        require(document.store->text == L"🙂 갑사", "real TSF composition preserves UTF-16 prefix");
        check(session->process({1, 0}, consumed), "composition backspace");
        require(document.store->text == L"🙂 갑ㅅ", "real TSF jamo deletion");
        check(session->process({2, 0}, consumed), "composition finish");
        require(document.store->text == L"🙂 갑ㅅ" && document.store->compositions == 0, "composition finish preserves text once");
        for (char key : std::string("rk")) check(session->process({0, static_cast<uint32_t>(key)}, consumed), "input before deferred finish");
        document.store->denyLock = true;
        session->finishLater();
        document.store->denyLock = false;
        for (char key : std::string("sk")) check(session->process({0, static_cast<uint32_t>(key)}, consumed), "input after deferred finish");
        check(session->process({2, 0}, consumed), "finish deferred input");
        require(document.store->text == L"🙂 갑ㅅ가나", "a refused finish never overwrites the previous word");
        for (char key : std::string("rk")) check(session->process({0, static_cast<uint32_t>(key)}, consumed), "input before host termination");
        ComPtr<ITfContextOwnerCompositionServices> owner;
        check(document.context.As(&owner), "context owner composition services");
        check(owner->TerminateComposition(nullptr), "host terminates composition");
        pump();
        for (char key : std::string("sk")) check(session->process({0, static_cast<uint32_t>(key)}, consumed), "input after host termination");
        check(session->process({2, 0}, consumed), "finish after host termination");
        require(document.store->text == L"🙂 갑ㅅ가나가나", "host termination never reuses committed preedit");
        session->retire(); session.Reset(); pump();
        check(thread->SetFocus(nullptr), "composition blur");
    }
    check(thread->Deactivate(), "composition thread deactivation");
}
}
int wmain(int argc, wchar_t** argv) {
    try {
        if (argc == 2 && std::wstring(argv[1]) == L"--core-unavailable") {
            CoreSession core(0);
            require(!core.available(), "missing or incompatible library must fail before entering Rust");
            std::cout << "PASS: unavailable core library is rejected without terminating the host" << std::endl;
            return 0;
        }
        HMODULE library = LoadLibraryExW(L"Amu.dll", nullptr, LOAD_LIBRARY_SEARCH_APPLICATION_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32);
        require(library != nullptr, "load Amu.dll");
        auto identity = reinterpret_cast<const char* (__cdecl*)()>(GetProcAddress(library, "AmuBuildJson"));
        require(identity != nullptr, "build identity export");
        if (argc == 2 && std::wstring(argv[1]) == L"--version") { std::cout << identity() << '\n'; FreeLibrary(library); return 0; }
        check(CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED), "COM apartment");
        auto getFactory = reinterpret_cast<HRESULT (__stdcall*)(REFCLSID, REFIID, void**)>(GetProcAddress(library, "DllGetClassObject"));
        require(getFactory != nullptr, "class factory export");
        ComPtr<IClassFactory> factory;
        bool registered = argc == 2 && (std::wstring(argv[1]) == L"--registered" || std::wstring(argv[1]) == L"--verify-registration");
        if (registered) {
            check(CoGetClassObject(ServiceId, CLSCTX_INPROC_SERVER, nullptr, IID_PPV_ARGS(&factory)), "registered COM class factory");
        } else {
            check(getFactory(ServiceId, IID_PPV_ARGS(&factory)), "class factory");
        }
        if (argc == 2 && std::wstring(argv[1]) == L"--verify-registration") {
            ComPtr<ITfTextInputProcessorEx> service;
            check(factory->CreateInstance(nullptr, IID_PPV_ARGS(&service)), "registered TSF service");
            service.Reset(); factory.Reset(); CoUninitialize(); FreeLibrary(library);
            std::cout << "PASS: registered COM text service loads" << std::endl;
            return 0;
        }
        testCoreAndKeys();
        testCompositionContext();
        if (argc == 2 && std::wstring(argv[1]) == L"--registered") testRegisteredTsf();
        factory.Reset();
        CoUninitialize();
        require(moduleObjects == 0, "all test documents and queued edit sessions must be released");
        std::cout << "PASS: Windows C ABI, key translation and TSF context checks\n";
        FreeLibrary(library);
        return 0;
    } catch (const std::exception& error) { std::cerr << "FAIL: " << error.what() << '\n'; return 1; }
}
