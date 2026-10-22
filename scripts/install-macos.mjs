import { existsSync, readFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { installBundle } from './install-bundle.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const SOURCE = resolve(ROOT, 'dist/Amu.app');
const DESTINATION = resolve(homedir(), 'Library/Input Methods/Amu.app');
const HELPER = resolve(ROOT, 'dist/amu-input-source');

main();

function main() {
  if (process.platform !== 'darwin') throw new Error('macOS에서 실행하세요.');
  if (!existsSync(SOURCE)) throw new Error('먼저 mise run macos:build를 실행하세요.');
  verifyBundle(SOURCE);
  if (existsSync(DESTINATION)) verifyBundle(DESTINATION);
  const build = JSON.parse(readFileSync(resolve(SOURCE, 'Contents/Resources/build.json'), 'utf8'));
  const result = installBundle({
    source: SOURCE,
    destination: DESTINATION,
    backupDirectory: resolve(homedir(), 'Library/Application Support/Amu/backups'),
    stop: () => run(HELPER, ['stop', DESTINATION]),
    activate: () => {
      run(HELPER, ['register', DESTINATION], [0, 2]);
      run('open', ['-g', DESTINATION]);
      run(HELPER, ['verify', DESTINATION, build.identifier]);
    },
  });
  if (result.backup) console.log(`이전 앱 백업: ${result.backup}`);
  console.log(`설치·실행 확인: ${build.identifier}\n경로: ${DESTINATION}`);
  console.log('현재 입력 소스는 유지했습니다. 입력 메뉴에서 Amu를 선택하세요.');
}

function verifyBundle(bundle) {
  const identifier = run('plutil', ['-extract', 'CFBundleIdentifier', 'raw', resolve(bundle, 'Contents/Info.plist')], [0], false).trim();
  if (identifier !== 'dev.amu.inputmethod.Amu') throw new Error(`다른 앱이므로 교체하지 않습니다: ${bundle}`);
  run('codesign', ['--verify', '--deep', '--strict', bundle]);
}

function run(command, args, allowedStatuses = [0], display = true) {
  const result = spawnSync(command, args, { encoding: 'utf8' });
  if (display && result.stdout) process.stdout.write(result.stdout);
  if (result.stderr) process.stderr.write(result.stderr);
  if (result.error) throw result.error;
  if (!allowedStatuses.includes(result.status)) throw new Error(`${command} 실패 (${result.status})`);
  return result.stdout;
}
