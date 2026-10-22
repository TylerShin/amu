import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

export function readReleaseAssets(directory, { commit, runNumber }) {
  const packages = ['macos', 'windows'].map((platform) => {
    const folder = resolve(directory, `${platform}-release`);
    const release = JSON.parse(readFileSync(resolve(folder, 'release.json'), 'utf8'));
    const expectedTag = `v${release.version}-dev.${runNumber}.${commit.slice(0, 12)}`;
    if (release.commit !== commit || release.dirty || release.tag !== expectedTag) throw new Error(`${platform}: 검사한 CI 커밋·실행과 다릅니다.`);
    const filename = platform === 'macos' ? /^Amu-v[\w.-]+-macos-(arm64|x86_64)\.zip$/ : /^Amu-v[\w.-]+-windows-x64\.zip$/;
    if (!filename.test(release.filename)) throw new Error(`${platform}: 잘못된 파일 이름입니다.`);
    const archive = resolve(folder, release.filename);
    const digest = createHash('sha256').update(readFileSync(archive)).digest('hex');
    if (digest !== release.sha256 || readFileSync(resolve(folder, 'SHA256SUMS'), 'utf8') !== `${digest}  ${release.filename}\n`) throw new Error(`${platform}: 패키지 체크섬이 다릅니다.`);
    return { ...release, platform, archive };
  });
  if (packages[0].tag !== packages[1].tag) throw new Error('플랫폼별 버전이 다릅니다.');
  return { version: packages[0].version, tag: packages[0].tag, commit, packages };
}
