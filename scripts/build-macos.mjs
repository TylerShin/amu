import { spawnSync } from 'node:child_process';
import { cpSync, existsSync, mkdirSync, readdirSync, renameSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { readBuildIdentity } from './build-identity.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const DIST = resolve(ROOT, 'dist');

main();

function main() {
  if (process.platform !== 'darwin') throw new Error('macOS is required for the AppKit adapter');
  const build = readBuildIdentity(ROOT);
  const messages = run('cargo', ['build', '--release', '--locked', '--message-format=json']).split('\n').filter(Boolean).map(JSON.parse);
  const native = messages.find((message) => message.reason === 'build-script-executed' && existsSync(resolve(message.out_dir, 'libhangul.dylib')));
  if (!native) throw new Error('Cargo did not report the libhangul build directory');
  mkdirSync(DIST, { recursive: true });
  const executable = resolve(DIST, 'Amu');
  const buildSource = resolve(DIST, 'AmuBuild.swift');
  writeFileSync(buildSource, `enum AmuBuild { static let json = ${JSON.stringify(JSON.stringify(build))} }\n`);
  const runtimeProtocol = resolve(ROOT, 'macos/RuntimeProtocol.swift');
  const sources = readdirSync(resolve(ROOT, 'macos/Sources')).filter((file) => file.endsWith('.swift')).map((file) => resolve(ROOT, 'macos/Sources', file));
  run('xcrun', ['swiftc', '-swift-version', '5', '-O', '-target', `${process.arch === 'arm64' ? 'arm64' : 'x86_64'}-apple-macosx13.0`,
    ...sources, runtimeProtocol, buildSource, '-import-objc-header', resolve(ROOT, 'include/amu.h'),
    '-L', resolve(ROOT, 'target/release'), '-lamu_ffi', '-L', native.out_dir, '-lhangul',
    '-framework', 'AppKit', '-framework', 'InputMethodKit',
    '-Xlinker', '-rpath', '-Xlinker', '@executable_path/../Frameworks', '-o', executable]);
  const library = resolve(native.out_dir, 'libhangul.dylib');
  run('install_name_tool', ['-change', library, '@rpath/libhangul.dylib', executable]);
  for (const playground of [false, true]) buildBundle(executable, library, playground, build);
  run('xcrun', ['swiftc', '-swift-version', '5', '-target', `${process.arch === 'arm64' ? 'arm64' : 'x86_64'}-apple-macosx13.0`,
    resolve(ROOT, 'macos/InputSourceTool.swift'), resolve(ROOT, 'macos/InstallationRuntime.swift'), runtimeProtocol,
    '-o', resolve(DIST, 'amu-input-source')]);
  console.log('Built dist/Amu.app and dist/Amu Playground.app');
}

function buildBundle(executable, library, playground, build) {
  const name = playground ? 'Amu Playground.app' : 'Amu.app';
  const buildID = Date.now().toString();
  const bundle = resolve(DIST, '.staging', buildID, name);
  const contents = resolve(bundle, 'Contents');
  for (const directory of ['MacOS', 'Frameworks', 'Resources']) mkdirSync(resolve(contents, directory), { recursive: true });
  cpSync(executable, resolve(contents, 'MacOS/Amu'));
  cpSync(library, resolve(contents, 'Frameworks/libhangul.dylib'));
  run('install_name_tool', ['-id', '@rpath/libhangul.dylib', resolve(contents, 'Frameworks/libhangul.dylib')]);
  cpSync(resolve(ROOT, 'third_party/libhangul'), resolve(contents, 'Resources/libhangul-source'), { recursive: true });
  cpSync(resolve(ROOT, 'docs/libhangul-rebuild.md'), resolve(contents, 'Resources/libhangul-rebuild.md'));
  cpSync(resolve(ROOT, 'models/initials'), resolve(contents, 'Resources/initial-model'), { recursive: true });
  cpSync(resolve(ROOT, 'scripts/train-initial-model.mjs'), resolve(contents, 'Resources/initial-model/train-initial-model.mjs'));
  cpSync(resolve(ROOT, 'models/korean'), resolve(contents, 'Resources/korean-model'), { recursive: true });
  cpSync(resolve(ROOT, 'scripts/train-korean-model.mjs'), resolve(contents, 'Resources/korean-model/train-korean-model.mjs'));
  cpSync(resolve(ROOT, 'LICENSE'), resolve(contents, 'Resources/Amu-LICENSE.txt'));
  cpSync(resolve(ROOT, 'THIRD_PARTY_NOTICES.md'), resolve(contents, 'Resources/THIRD_PARTY_NOTICES.md'));
  const info = {
    CFBundleIdentifier: playground ? 'dev.amu.Playground' : 'dev.amu.inputmethod.Amu',
    CFBundleName: playground ? 'Amu Playground' : 'Amu',
    CFBundleDisplayName: playground ? 'Amu Playground' : 'Amu',
    CFBundleExecutable: 'Amu', CFBundlePackageType: 'APPL',
    CFBundleVersion: build.buildNumber, CFBundleShortVersionString: build.version,
    AmuBuildIdentifier: build.identifier,
    NSPrincipalClass: 'NSApplication', NSHighResolutionCapable: true,
    LSMinimumSystemVersion: '13.0', LSUIElement: !playground,
  };
  if (!playground) Object.assign(info, {
    InputMethodConnectionName: 'dev.amu.inputmethod.Amu.Connection',
    InputMethodServerControllerClass: 'AmuInputController',
    TISInputSourceID: 'dev.amu.inputmethod.Amu',
    TISIntendedLanguage: 'ko',
    ComponentInputModeDict: { tsInputModeListKey: { 'dev.amu.inputmethod.Amu.auto': {
      TISInputSourceID: 'dev.amu.inputmethod.Amu.auto', TISIntendedLanguage: 'ko',
      tsInputModeDefaultStateKey: true, tsInputModeIsVisibleKey: true,
      tsInputModePrimaryInScriptKey: true, tsInputModeScriptKey: 'smKorean',
      tsInputModeCharacterRepertoireKey: ['Latn', 'Hang'],
    } } },
    tsInputMethodCharacterRepertoireKey: ['Latn', 'Hang'],
    tsInputMethodPrimaryInScriptKey: true, tsInputMethodScriptKey: 'smKorean',
  });
  run('plutil', ['-convert', 'xml1', '-o', resolve(contents, 'Info.plist'), '-'], JSON.stringify(info));
  for (const locale of ['en', 'ko']) {
    const localization = resolve(contents, 'Resources', `${locale}.lproj`);
    mkdirSync(localization, { recursive: true });
    writeFileSync(resolve(localization, 'InfoPlist.strings'), '"dev.amu.inputmethod.Amu.auto" = "Amu";\n');
  }
  writeFileSync(resolve(contents, 'Resources/build.json'), JSON.stringify(build, null, 2) + '\n');
  writeFileSync(resolve(contents, 'Resources/ABOUT.txt'), `Amu · 아무 — 아무거나 쓰세요.\n${build.identifier}\nhttps://github.com/TylerShin/amu\nAmu 코드: MIT (Amu-LICENSE.txt).\nlibhangul: LGPL-2.1-or-later. 원본 소스·라이선스·교체 방법을 포함합니다.\n한국어 음절 통계: UD Korean GSD에서 변환, CC BY-SA 4.0. 출처·통계·재생성 스크립트는 korean-model/에 있습니다.\n외부 구성 요소의 상세 출처: THIRD_PARTY_NOTICES.md 및 프로젝트 저장소.\n`);
  run('codesign', ['--force', '--sign', '-', resolve(contents, 'Frameworks/libhangul.dylib')]);
  run('codesign', ['--force', '--sign', '-', bundle]);
  run('codesign', ['--verify', '--deep', '--strict', bundle]);
  const destination = resolve(DIST, name);
  if (existsSync(destination)) {
    const previous = resolve(DIST, '.previous', buildID);
    mkdirSync(previous, { recursive: true });
    renameSync(destination, resolve(previous, name));
  }
  renameSync(bundle, destination);
}

function run(command, args, input) {
  const result = spawnSync(command, args, { cwd: ROOT, input, encoding: 'utf8', maxBuffer: 8 * 1024 * 1024 });
  if (result.stderr) process.stderr.write(result.stderr);
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} failed (${result.status})`);
  return result.stdout;
}
