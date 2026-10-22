import { execFileSync } from 'node:child_process';
import { randomUUID } from 'node:crypto';

export function readBuildIdentity(root) {
  const git = (args) => execFileSync('git', args, { cwd: root, encoding: 'utf8' }).trim();
  const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--no-deps', '--locked', '--format-version=1'], { cwd: root, encoding: 'utf8' }));
  const version = metadata.packages.find((item) => item.name === 'amu-ffi').version;
  const commit = git(['rev-parse', 'HEAD']);
  const dirty = git(['status', '--porcelain', '--untracked-files=normal']) !== '';
  const runNumber = process.env.GITHUB_RUN_NUMBER;
  if (runNumber && (!/^\d+$/.test(runNumber) || dirty || process.env.GITHUB_SHA !== commit)) {
    throw new Error('CI builds require a clean checkout of GITHUB_SHA and a numeric run number');
  }
  const channel = runNumber ? `dev.${runNumber}` : 'local';
  const tag = `v${version}-${channel}.${commit.slice(0,12)}`;
  const identifier = runNumber ? tag : `${tag}${dirty ? '-dirty' : ''}.${randomUUID()}`;
  return { version, commit, dirty, identifier, tag, buildNumber: runNumber ?? '1', architecture: process.arch, minimumOS: '13.0', runtimeProtocol: 1 };
}
