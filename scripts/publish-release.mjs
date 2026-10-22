import { execFileSync, spawnSync } from 'node:child_process';
import { cpSync, mkdirSync, writeFileSync } from 'node:fs';
import { readReleaseAssets } from './release-assets.mjs';
import { resolve } from 'node:path';

main();

function main() {
  const directory = resolve('dist/release');
  const repository = process.env.GITHUB_REPOSITORY;
  if (repository !== 'TylerShin/amu' || process.env.GITHUB_REF !== 'refs/heads/main') throw new Error('Releases are published only from TylerShin/amu main');
  if (!['push', 'workflow_dispatch'].includes(process.env.GITHUB_EVENT_NAME)) throw new Error('This event cannot publish releases');
  const release = readReleaseAssets(resolve('dist/release-input'), { commit: process.env.GITHUB_SHA, runNumber: process.env.GITHUB_RUN_NUMBER });
  mkdirSync(directory, { recursive: true });
  for (const item of release.packages) cpSync(item.archive, resolve(directory, item.filename));
  writeFileSync(resolve(directory, 'SHA256SUMS'), release.packages.map((item) => `${item.sha256}  ${item.filename}\n`).join(''));
  writeFileSync(resolve(directory, 'release.json'), JSON.stringify({ ...release, packages: release.packages.map(({ archive, ...item }) => item) }, null, 2) + '\n');
  writeFileSync(resolve(directory, 'release-notes.md'), releaseNotes(release));
  const assets = [...release.packages.map((item) => item.filename), 'SHA256SUMS', 'release.json'];
  ensureTag(repository, release.tag, release.commit);
  const existing = spawnSync('gh', ['release', 'view', release.tag, '--repo', repository, '--json', 'isDraft,targetCommitish,assets'], { encoding: 'utf8' });
  if (existing.status === 0) {
    const previous = JSON.parse(existing.stdout);
    if (previous.targetCommitish !== release.commit) throw new Error('Existing release targets another commit');
    if (!previous.isDraft) {
      const taggedCommit = gh(['api', `repos/${repository}/commits/${release.tag}`, '--jq', '.sha']).trim();
      if (taggedCommit !== release.commit || !assets.every((name) => previous.assets.some((asset) => asset.name === name))) {
        throw new Error('Published release is incomplete or points to another commit; not overwriting it');
      }
      console.log(`이미 공개된 릴리스 유지: ${release.tag}`);
      return;
    }
  } else {
    gh(['release', 'create', release.tag, '--repo', repository, '--verify-tag', '--target', release.commit, '--draft', '--prerelease', '--title', `Amu ${release.tag}`, '--notes-file', resolve(directory, 'release-notes.md')]);
  }
  gh(['release', 'upload', release.tag, '--repo', repository, '--clobber', ...assets.map((name) => resolve(directory, name))]);
  gh(['release', 'edit', release.tag, '--repo', repository, '--draft=false', '--prerelease', '--latest=false']);
  console.log(`릴리스 공개: https://github.com/${repository}/releases/tag/${release.tag}`);
}

function ensureTag(repository, tag, commit) {
  const refs = JSON.parse(gh(['api', `repos/${repository}/git/matching-refs/tags/${tag}`]));
  if (refs.some((ref) => ref.ref === `refs/tags/${tag}`)) {
    const existing = gh(['api', `repos/${repository}/commits/${tag}`, '--jq', '.sha']).trim();
    if (existing !== commit) throw new Error('Existing tag points to another commit');
    return;
  }
  gh(['api', '--method', 'POST', `repos/${repository}/git/refs`, '-f', `ref=refs/tags/${tag}`, '-f', `sha=${commit}`]);
}

function gh(args) {
  return execFileSync('gh', args, { encoding: 'utf8' });
}

function releaseNotes(release) {
  return `# Amu ${release.tag}

같은 커밋의 Linux·macOS·Windows CI를 통과한 실험 버전입니다.

- 소스 커밋: ${release.commit}
- macOS: Apple Silicon / macOS 13 이상. ad-hoc 서명이며 Apple 공증은 없습니다.
- Windows: Windows 11 x64. 64비트·32비트 앱용 TSF DLL을 함께 포함하며 인증서 서명은 없습니다.
- 패키지: ${release.packages.map((item) => '`' + item.filename + '`').join(', ')}

[macOS 설치 안내](https://github.com/TylerShin/amu/blob/${release.commit}/docs/install-release.md) · [Windows 설치 안내](https://github.com/TylerShin/amu/blob/${release.commit}/docs/install-windows.md)

Windows는 ZIP을 풀고 관리자 PowerShell에서 Install.ps1을 실행한 뒤 로그아웃·로그인하고 Win+Space로 Amu를 선택합니다. 이미 실행 중인 앱의 DLL을 강제 교체하지 않습니다. macOS 다운로드 앱은 Gatekeeper 승인이 필요할 수 있습니다.

자동 판독과 외부 앱별 IME 호환성은 실험 단계입니다. 코어·네이티브 호스트 검사를 통과한 사실이 모든 앱의 호환성을 보장하지는 않습니다. SHA256SUMS로 다운로드한 ZIP의 무결성을 확인할 수 있습니다. 두 패키지에 라이선스·libhangul 원본 및 교체 안내·모델 재생성 자료를 포함했습니다.

[변경 기록](https://github.com/TylerShin/amu/blob/${release.commit}/CHANGELOG.md) · [CI 실행](https://github.com/TylerShin/amu/actions/runs/${process.env.GITHUB_RUN_ID ?? ''})
`;
}
