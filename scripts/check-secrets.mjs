import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, sep } from 'node:path';

main();

function main() {
  const directory = mkdtempSync(join(tmpdir(), 'amu-index-'));
  try {
    run('git', ['checkout-index', '--all', `--prefix=${directory}${sep}`]);
    run('gitleaks', ['dir', '--redact', '--no-banner', directory]);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

function run(command, args) {
  const result = spawnSync(command, args, { stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} failed (${result.status})`);
}
