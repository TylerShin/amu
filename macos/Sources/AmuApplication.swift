import AppKit
import InputMethodKit

@main
@MainActor
struct AmuApplication {
    static func main() {
        if CommandLine.arguments.contains("--version") {
            print(AmuBuild.json)
            return
        }
        let application = NSApplication.shared
        let isPlayground = Bundle.main.bundleIdentifier == "dev.amu.Playground"
        var playground: Playground?
        var inputServer: IMKServer?
        var runtime: RuntimeStatus?

        if isPlayground {
            application.setActivationPolicy(.regular)
            installApplicationMenu(application)
            playground = Playground()
            playground?.show()
        } else {
            application.setActivationPolicy(.accessory)
            guard let name = Bundle.main.object(forInfoDictionaryKey: "InputMethodConnectionName") as? String,
                  let identifier = Bundle.main.bundleIdentifier else { fatalError("Missing input method bundle metadata") }
            inputServer = IMKServer(name: name, bundleIdentifier: identifier)
            guard inputServer != nil else { fatalError("Could not start input method server") }
            guard NSClassFromString("AmuInputController") != nil else { fatalError("Missing Objective-C controller registration") }
            if CommandLine.arguments.contains("--check-server") {
                print("InputMethodKit server and AmuInputController registration: passed")
                return
            }
            runtime = RuntimeStatus()
        }

        withExtendedLifetime((playground, inputServer, runtime)) { application.run() }
    }

    private static func installApplicationMenu(_ application: NSApplication) {
        let menu = NSMenu()
        let appItem = NSMenuItem()
        let appMenu = NSMenu()
        appMenu.addItem(withTitle: "Amu 종료", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")
        appItem.submenu = appMenu
        menu.addItem(appItem)
        let editItem = NSMenuItem()
        let editMenu = NSMenu(title: "편집")
        editMenu.addItem(withTitle: "잘라내기", action: #selector(NSText.cut(_:)), keyEquivalent: "x")
        editMenu.addItem(withTitle: "복사", action: #selector(NSText.copy(_:)), keyEquivalent: "c")
        editMenu.addItem(withTitle: "붙여넣기", action: #selector(NSText.paste(_:)), keyEquivalent: "v")
        editMenu.addItem(withTitle: "전체 선택", action: #selector(NSText.selectAll(_:)), keyEquivalent: "a")
        editItem.submenu = editMenu
        menu.addItem(editItem)
        application.mainMenu = menu
    }
}
