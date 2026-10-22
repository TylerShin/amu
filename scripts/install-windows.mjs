import { execFileSync } from 'node:child_process';
import { readdirSync } from 'node:fs';
import { resolve } from 'node:path';

const packages = readdirSync('dist/windows-package', { withFileTypes: true }).filter((entry) => entry.isDirectory());
const build = JSON.parse(execFileSync(resolve('dist/windows/x64/amu-check.exe'), ['--version'], { encoding: 'utf8' }));
const name = `Amu-${build.tag}-windows-x64`;
if (!packages.some((entry) => entry.name === name)) throw new Error('현재 빌드에 맞는 Windows 패키지가 없습니다.');
execFileSync('powershell.exe', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', resolve('dist/windows-package', name, 'Install.ps1')], { stdio: 'inherit' });
