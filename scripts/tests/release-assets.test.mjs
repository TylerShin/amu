import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { test } from 'node:test';
import { readReleaseAssets } from '../release-assets.mjs';

const commit = '1234567890ab1234567890ab1234567890ab12345678';
const runNumber = '9';
const tag = 'v0.1.0-dev.9.1234567890ab';

test('두 플랫폼 모두 같은 CI 커밋과 체크섬이어야 공개한다', (t) => {
  const directory = fixture(t);
  const result = readReleaseAssets(directory, { commit, runNumber });
  assert.equal(result.tag, tag);
  assert.equal(result.packages.length, 2);
});

test('Windows 패키지가 없거나 다른 실행의 패키지이면 공개하지 않는다', (t) => {
  const directory = fixture(t);
  assert.throws(() => readReleaseAssets(directory, { commit, runNumber: '10' }), /CI/);
  rmSync(resolve(directory, 'windows-release'), { recursive: true });
  assert.throws(() => readReleaseAssets(directory, { commit, runNumber }), /ENOENT/);
});

test('ZIP이 변조되면 공개하지 않는다', (t) => {
  const directory = fixture(t);
  writeFileSync(resolve(directory, 'windows-release', `Amu-${tag}-windows-x64.zip`), 'tampered');
  assert.throws(() => readReleaseAssets(directory, { commit, runNumber }), /체크섬/);
});

function fixture(t) {
  const directory = mkdtempSync(resolve(tmpdir(), 'amu-release-'));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  for (const [platform, architecture] of [['macos', 'arm64'], ['windows', 'x64']]) {
    const folder = resolve(directory, `${platform}-release`);
    mkdirSync(folder);
    const filename = `Amu-${tag}-${platform}-${architecture}.zip`;
    const contents = Buffer.from(platform);
    const sha256 = createHash('sha256').update(contents).digest('hex');
    writeFileSync(resolve(folder, filename), contents);
    writeFileSync(resolve(folder, 'SHA256SUMS'), `${sha256}  ${filename}\n`);
    writeFileSync(resolve(folder, 'release.json'), JSON.stringify({ version: '0.1.0', tag, commit, dirty: false, filename, sha256 }));
  }
  return directory;
}
