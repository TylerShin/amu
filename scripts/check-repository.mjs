import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

main();

function main() {
  const files = run('git', ['ls-files', '-z']).split('\0').filter(Boolean);
  if (files.length === 0) throw new Error('Stage the public files with git add before checking');
  const ignored = run('git', ['ls-files', '-ci', '--exclude-standard']).trim();
  if (ignored) throw new Error(`Ignored files are tracked:\n${ignored}`);
  verifyModelChecksum();
  for (const file of files) {
    if (file.endsWith('.mjs')) run(process.execPath, ['--check', file]);
    if (file.endsWith('.md')) verifyLocalLinks(file);
  }
  console.log(`Repository checks passed (${files.length} tracked files)`);
}

function verifyModelChecksum() {
  for (const [directory, filename] of [['korean', 'syllables.tsv'], ['initials', 'pairs.tsv']]) {
    const manifest = JSON.parse(readFileSync(`models/${directory}/manifest.json`, 'utf8'));
    const actual = createHash('sha256').update(readFileSync(`models/${directory}/${filename}`)).digest('hex');
    if (actual !== manifest.modelSha256) throw new Error(`${directory} model checksum differs from manifest.json`);
  }
}

function verifyLocalLinks(file) {
  const markdown = readFileSync(file, 'utf8').replace(/```[\s\S]*?```/g, '');
  for (const match of markdown.matchAll(/\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g)) {
    const link = match[1];
    if (/^(?:[a-z][a-z\d+.-]*:|#|\/\/)/i.test(link)) continue;
    const target = decodeURIComponent(link.split(/[?#]/)[0]);
    if (!existsSync(resolve(dirname(file), target))) throw new Error(`Broken local link: ${file} → ${target}`);
  }
}

function run(command, args) {
  const result = spawnSync(command, args, { encoding: 'utf8' });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} failed (${result.status}): ${result.stderr}`);
  return result.stdout;
}
