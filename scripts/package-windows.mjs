import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

main();

function main() {
  if (process.platform !== 'win32') throw new Error('Windows에서 패키징하세요.');
  const build = JSON.parse(readFileSync('dist/windows/build.json', 'utf8'));
  if (process.env.GITHUB_SHA && (build.commit !== process.env.GITHUB_SHA || build.dirty)) throw new Error('CI 커밋과 패키지가 다릅니다.');
  const name = `Amu-${build.tag}-windows-x64`;
  const directory = resolve('dist/windows-package', name);
  rmSync(directory, { recursive: true, force: true });
  mkdirSync(directory, { recursive: true });
  for (const architecture of ['x64', 'x86']) {
    const source = resolve('dist/windows', architecture);
    const identity = JSON.parse(execFileSync(resolve(source, 'amu-check.exe'), ['--version'], { encoding: 'utf8' }));
    if (JSON.stringify(identity) !== JSON.stringify(build)) throw new Error('DLL 버전과 메타데이터가 다릅니다.');
    for (const file of readdirSync(source).filter((file) => /\.(dll|exe)$/.test(file))) {
      mkdirSync(resolve(directory, architecture), { recursive: true });
      cpSync(resolve(source, file), resolve(directory, architecture, file));
    }
  }
  for (const [source, target] of [
    ['windows/Install.ps1', 'Install.ps1'], ['docs/install-windows.md', 'README.md'], ['LICENSE', 'LICENSE'],
    ['THIRD_PARTY_NOTICES.md', 'THIRD_PARTY_NOTICES.md'], ['third_party/libhangul', 'libhangul-source'],
    ['models/korean', 'korean-model'], ['scripts/train-korean-model.mjs', 'korean-model/train-korean-model.mjs'],
    ['docs/libhangul-rebuild-windows.md', 'libhangul-rebuild.md'],
  ]) cpSync(source, resolve(directory, target), { recursive: true });
  // Windows PowerShell 5.1 needs a BOM to read Korean source as UTF-8.
  writeFileSync(resolve(directory, 'Install.ps1'), '\ufeff' + readFileSync('windows/Install.ps1', 'utf8'));
  const files = readdirSync(directory, { recursive: true, withFileTypes: true }).filter((entry) => entry.isFile()).map((entry) => {
    const file = resolve(entry.parentPath, entry.name);
    return { path: file.slice(directory.length + 1).replaceAll('\\', '/'), sha256: digest(file) };
  }).sort((a, b) => a.path.localeCompare(b.path));
  writeFileSync(resolve(directory, 'package.json'), JSON.stringify({ ...build, files }, null, 2) + '\n');
  // Exercise the shipped installer on an ephemeral CI machine, never the developer's PC.
  if (process.env.GITHUB_ACTIONS === 'true') {
    for (const action of ['Install', 'Status', 'Uninstall']) execFileSync('powershell.exe', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', resolve(directory, 'Install.ps1'), '-Action', action, '-SkipLanguageSetup'], { stdio: 'inherit' });
  }
  const output = resolve('dist/release-windows');
  mkdirSync(output, { recursive: true });
  const filename = `${name}.zip`;
  const archive = resolve(output, filename);
  if (existsSync(archive)) rmSync(archive);
  execFileSync('tar.exe', ['-a', '-c', '-f', archive, '-C', resolve('dist/windows-package'), name], { stdio: 'inherit' });
  const sha256 = digest(archive);
  writeFileSync(resolve(output, 'release.json'), JSON.stringify({ ...build, filename, sha256 }, null, 2) + '\n');
  writeFileSync(resolve(output, 'SHA256SUMS'), `${sha256}  ${filename}\n`);
  console.log(`Windows 패키지 완료: ${archive}`);
}
function digest(path) { return createHash('sha256').update(readFileSync(path)).digest('hex'); }
