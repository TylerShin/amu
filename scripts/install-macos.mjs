import { cpSync, existsSync, mkdirSync, renameSync } from 'node:fs';
import { homedir } from 'node:os';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

const source = resolve('dist/Amu.app');
const parent = resolve(homedir(), 'Library/Input Methods');
const destination = resolve(parent, 'Amu.app');
if (!existsSync(source)) throw new Error('Run mise run macos:build first');
mkdirSync(parent, { recursive: true });
if (existsSync(destination)) {
  const metadata = spawnSync('plutil', ['-extract', 'CFBundleIdentifier', 'raw', resolve(destination, 'Contents/Info.plist')], { encoding: 'utf8' });
  if (metadata.status !== 0 || metadata.stdout.trim() !== 'dev.amu.inputmethod.Amu') {
    throw new Error('An unrelated Amu.app exists; installation left it untouched');
  }
  const backupDirectory = resolve(homedir(), 'Library/Application Support/Amu/backups');
  mkdirSync(backupDirectory, { recursive: true });
  const backup = resolve(backupDirectory, `Amu-${Date.now()}.app`);
  renameSync(destination, backup);
  console.log(`Previous app preserved at ${backup}`);
}
cpSync(source, destination, { recursive: true });
const registration = spawnSync(resolve('dist/amu-input-source'), ['register', destination], { encoding: 'utf8' });
process.stdout.write(registration.stdout ?? '');
process.stderr.write(registration.stderr ?? '');
if (registration.error) throw registration.error;
if (registration.status !== 0 && registration.status !== 2) process.exit(registration.status ?? 1);
console.log(`Installed: ${destination}`);
console.log('Choose Amu from the macOS input menu. The current input source has not been changed.');
