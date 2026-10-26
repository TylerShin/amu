import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const BUNDLE = resolve(ROOT, 'dist/Amu.app');
const OUTPUT = resolve(ROOT, 'dist/release');

main();

function main() {
  const build = JSON.parse(readFileSync(resolve(BUNDLE, 'Contents/Resources/build.json'), 'utf8'));
  const embedded = JSON.parse(run(resolve(BUNDLE, 'Contents/MacOS/Amu'), ['--version']));
  if (JSON.stringify(embedded) !== JSON.stringify(build)) throw new Error('Bundle metadata differs from the compiled version');
  if (process.env.GITHUB_SHA && (build.commit !== process.env.GITHUB_SHA || build.dirty)) throw new Error('Package must match the clean CI commit');
  const architecture = build.architecture === 'arm64' ? 'arm64' : 'x86_64';
  const actual = run('lipo', ['-archs', resolve(BUNDLE, 'Contents/MacOS/Amu')]).trim();
  if (actual !== architecture) throw new Error(`Unexpected package architecture: ${actual}`);
  verifyBundle(BUNDLE);
  rmSync(OUTPUT, { recursive: true, force: true });
  mkdirSync(OUTPUT, { recursive: true });
  const staging = mkdtempSync(resolve(ROOT, 'dist/package-'));
  try {
    const name = `Amu-${build.tag}-macos-${architecture}`;
    const payload = resolve(staging, name);
    mkdirSync(payload);
    cpSync(BUNDLE, resolve(payload, 'Amu.app'), { recursive: true });
    cpSync(resolve(ROOT, 'docs/install-release.md'), resolve(payload, '설치안내.md'));
    const filename = `${name}.zip`;
    const archive = resolve(OUTPUT, filename);
    run('ditto', ['-c', '-k', '--sequesterRsrc', '--keepParent', payload, archive]);
    const unpacked = resolve(staging, 'verify');
    run('ditto', ['-x', '-k', archive, unpacked]);
    verifyBundle(resolve(unpacked, name, 'Amu.app'));
    const sha256 = createHash('sha256').update(readFileSync(archive)).digest('hex');
    writeFileSync(resolve(OUTPUT, 'SHA256SUMS'), `${sha256}  ${filename}\n`);
    writeFileSync(resolve(OUTPUT, 'release.json'), JSON.stringify({ ...build, filename, sha256 }, null, 2) + '\n');
    writeFileSync(resolve(OUTPUT, 'release-notes.md'), releaseNotes(build, filename));
    console.log(`패키지 검사 완료: ${archive}\nSHA-256: ${sha256}`);
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
}

function verifyBundle(bundle) {
  run('codesign', ['--verify', '--deep', '--strict', bundle]);
  for (const file of ['Amu-LICENSE.txt', 'THIRD_PARTY_NOTICES.md', 'libhangul-source/COPYING', 'libhangul-rebuild.md', 'initial-model/manifest.json', 'initial-model/pairs.tsv', 'initial-model/LICENSE-korean.txt', 'initial-model/LICENSE-english.txt', 'initial-model/train-initial-model.mjs', 'korean-model/manifest.json', 'korean-model/train-korean-model.mjs', 'build.json']) {
    if (!existsSync(resolve(bundle, 'Contents/Resources', file))) throw new Error(`Missing package resource: ${file}`);
  }
}

function releaseNotes(build, filename) {
  return `# Amu ${build.tag}\n\nmain의 Linux·macOS CI를 통과한 실험 버전입니다.\n\n- 소스 커밋: ${build.commit}\n- 빌드: ${build.identifier}\n- 대상: macOS ${build.minimumOS} 이상 / ${build.architecture === 'arm64' ? 'Apple Silicon' : 'Intel'}\n- 패키지: \`${filename}\`\n- 서명: ad-hoc. Developer ID 인증서와 Apple 공증은 아직 적용하지 않았습니다.\n\n다운로드한 앱은 Gatekeeper 승인이 필요할 수 있습니다. [설치·업데이트 안내](https://github.com/TylerShin/amu/blob/${build.commit}/docs/install-release.md)를 먼저 확인하세요. 외부 앱별 IME 호환성과 최소 OS에서의 실사용은 추가 검증 중입니다.\n\nZIP과 SHA256SUMS를 같은 폴더에 받은 뒤 \`shasum -a 256 -c SHA256SUMS\`로 무결성을 확인할 수 있습니다. 앱에 Amu MIT, libhangul 원본·LGPL·교체 안내, 모델 출처·라이선스·재생성 자료를 포함했습니다.\n\n[변경 기록](https://github.com/TylerShin/amu/blob/${build.commit}/CHANGELOG.md) · [CI 실행](https://github.com/TylerShin/amu/actions/runs/${process.env.GITHUB_RUN_ID ?? ''})\n`;
}

function run(command, args) {
  return execFileSync(command, args, { cwd: ROOT, encoding: 'utf8' });
}
