import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

main();

function main() {
  const directory = resolve('dist/release');
  const release = JSON.parse(readFileSync(resolve(directory, 'release.json'), 'utf8'));
  const repository = process.env.GITHUB_REPOSITORY;
  if (repository !== 'TylerShin/amu' || process.env.GITHUB_REF !== 'refs/heads/main') throw new Error('Releases are published only from TylerShin/amu main');
  if (!['push', 'workflow_dispatch'].includes(process.env.GITHUB_EVENT_NAME)) throw new Error('This event cannot publish releases');
  if (release.commit !== process.env.GITHUB_SHA || release.dirty || !/^v\d+\.\d+\.\d+-dev\.\d+\.[a-f0-9]{12}$/.test(release.tag)) {
    throw new Error('Release metadata does not match the tested CI commit');
  }
  const expectedTag = `v${release.version}-dev.${process.env.GITHUB_RUN_NUMBER}.${release.commit.slice(0, 12)}`;
  if (release.tag !== expectedTag) throw new Error('Artifact belongs to a different workflow run');
  // 아티팩트 안의 경로를 실행 인자나 파일 시스템 경로로 무제한 신뢰하지 않는다.
  if (!/^Amu-v[\w.-]+-macos-(arm64|x86_64)\.zip$/.test(release.filename)) throw new Error('Unexpected asset filename');
  const archive = resolve(directory, release.filename);
  const actual = createHash('sha256').update(readFileSync(archive)).digest('hex');
  if (actual !== release.sha256 || readFileSync(resolve(directory, 'SHA256SUMS'), 'utf8') !== `${actual}  ${release.filename}\n`) {
    throw new Error('Release archive checksum mismatch');
  }
  const assets = [release.filename, 'SHA256SUMS', 'release.json'];
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
