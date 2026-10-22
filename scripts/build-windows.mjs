import { cpSync, existsSync, mkdirSync, readdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { readBuildIdentity } from './build-identity.mjs';
import { runWindows, windowsEnvironment } from './windows-toolchain.mjs';

const ROOT = process.cwd();
main();

function main() {
  const build = { ...readBuildIdentity(ROOT), platform: 'windows', architecture: 'x64+x86', minimumOS: 'Windows 11 x64' };
  const directory = resolve(ROOT, 'dist/windows');
  mkdirSync(directory, { recursive: true });
  writeFileSync(resolve(directory, 'build_identity.h'), `#define AMU_BUILD_JSON ${JSON.stringify(JSON.stringify(build))}\n`);
  for (const architecture of ['x64', 'x86']) buildArchitecture(architecture, directory);
  writeFileSync(resolve(directory, 'build.json'), JSON.stringify(build, null, 2) + '\n');
  console.log(`Windows 입력기 빌드 완료: ${build.identifier}`);
}

function buildArchitecture(architecture, directory) {
  const env = windowsEnvironment(architecture);
  const target = architecture === 'x64' ? 'x86_64-pc-windows-msvc' : 'i686-pc-windows-msvc';
  runWindows('rustup', ['target', 'add', target], { env });
  const output = runWindows('cargo', ['build', '--release', '--locked', '--target', target, '--message-format=json'], { env, capture: true });
  const messages = output.split(/\r?\n/).filter((line) => line.startsWith('{')).map(JSON.parse);
  const native = messages.find((message) => message.reason === 'build-script-executed' && existsSync(resolve(message.out_dir, 'hangul.dll')));
  if (!native) throw new Error('Cargo의 libhangul DLL 출력 경로를 찾지 못했습니다.');
  const destination = resolve(directory, architecture);
  mkdirSync(destination, { recursive: true });
  cpSync(resolve(native.out_dir, 'hangul.dll'), resolve(destination, 'hangul.dll'));
  const sources = readdirSync(resolve(ROOT, 'windows/src')).filter((file) => file.endsWith('.cpp')).map((file) => resolve(ROOT, 'windows/src', file));
  const flags = ['/nologo', '/std:c++17', '/EHsc', '/O2', '/MD', '/W4', '/WX', '/utf-8', '/DUNICODE', '/D_UNICODE', '/D_WIN32_WINNT=0x0A00', `/I${resolve(ROOT, 'include')}`, `/I${resolve(ROOT, 'windows/src')}`, `/I${directory}`];
  const libraries = [resolve(ROOT, 'target', target, 'release/amu_ffi.lib'), resolve(native.out_dir, 'hangul.lib'),
    'ole32.lib', 'oleaut32.lib', 'uuid.lib', 'user32.lib', 'advapi32.lib', 'delayimp.lib',
    'ws2_32.lib', 'userenv.lib', 'bcrypt.lib', 'ntdll.lib', 'synchronization.lib'];
  runWindows('cl.exe', [...flags, '/LD', ...sources, '/Fe:Amu.dll', '/link', `/DEF:${resolve(ROOT, 'windows/Amu.def')}`, '/DELAYLOAD:hangul.dll', ...libraries], { env, cwd: destination });
  const testSources = ['windows/tests/smoke.cpp', 'windows/tests/text_store.cpp', 'windows/src/core_session.cpp', 'windows/src/composition.cpp', 'windows/src/context_session.cpp', 'windows/src/key_input.cpp', 'windows/src/dependency_loader.cpp'].map((file) => resolve(ROOT, file));
  runWindows('cl.exe', [...flags, ...testSources, '/Fe:amu-check.exe', '/link', '/DELAYLOAD:hangul.dll', ...libraries], { env, cwd: destination });
  // Runtime DLLs are copied app-locally; a home PC does not need Visual Studio.
  const redistRoot = resolve(env.VCToolsRedistDir, architecture);
  const runtime = readdirSync(redistRoot).find((name) => /^Microsoft\.VC\d+\.CRT$/.test(name));
  if (!runtime) throw new Error(`MSVC 런타임을 찾지 못했습니다: ${redistRoot}`);
  const redist = resolve(redistRoot, runtime);
  for (const file of readdirSync(redist).filter((name) => name.endsWith('.dll'))) cpSync(resolve(redist, file), resolve(destination, file));
  runWindows(resolve(destination, 'amu-check.exe'), [], { env, cwd: destination });
}
