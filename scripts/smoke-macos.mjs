import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';

const sources = ['CoreSession', 'KeyTranslator', 'InputCoordinator', 'Playground'].map((name) => `macos/Sources/${name}.swift`);
run('xcrun', ['swiftc', '-swift-version', '5', '-O', ...sources, 'macos/SmokeTest.swift',
  '-import-objc-header', 'include/amu.h', '-L', 'target/release', '-lamu_ffi',
  '-L', 'dist/Amu.app/Contents/Frameworks', '-lhangul', '-framework', 'AppKit',
  '-Xlinker', '-rpath', '-Xlinker', '@executable_path/Amu.app/Contents/Frameworks', '-o', 'dist/amu-smoke']);
run(resolve('dist/amu-smoke'), []);
run(resolve('dist/Amu.app/Contents/MacOS/Amu'), ['--check-server']);

function run(command, args) {
  const result = spawnSync(command, args, { stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
