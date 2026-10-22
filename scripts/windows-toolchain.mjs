import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { resolve } from 'node:path';

export function windowsEnvironment(architecture) {
  if (process.platform !== 'win32') throw new Error('이 명령은 Windows에서 실행하세요.');
  if (!['x64', 'x86'].includes(architecture)) throw new Error('지원하는 빌드 아키텍처는 x64, x86입니다.');
  const vswhere = resolve(process.env['ProgramFiles(x86)'], 'Microsoft Visual Studio/Installer/vswhere.exe');
  if (!existsSync(vswhere)) throw new Error('Visual Studio Build Tools의 C++ 데스크톱 개발과 Windows SDK를 설치하세요.');
  const installation = execFileSync(vswhere, ['-latest', '-products', '*', '-requires', 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64', '-property', 'installationPath'], { encoding: 'utf8' }).trim();
  if (!installation || /["%\r\n]/.test(installation)) throw new Error('유효한 MSVC 설치를 찾지 못했습니다.');
  const command = `""${resolve(installation, 'Common7/Tools/VsDevCmd.bat')}" -no_logo -arch=${architecture} -host_arch=x64 >nul && set"`;
  const variables = execFileSync(process.env.ComSpec, ['/d', '/s', '/c', command], { encoding: 'utf8', windowsVerbatimArguments: true });
  const environment = { ...process.env };
  for (const line of variables.split(/\r?\n/)) {
    const index = line.indexOf('=');
    if (index <= 0) continue;
    const name = line.slice(0, index);
    for (const existing of Object.keys(environment)) if (existing.toLowerCase() === name.toLowerCase()) delete environment[existing];
    environment[name] = line.slice(index + 1);
  }
  return environment;
}

export function runWindows(command, args, { env, cwd = process.cwd(), capture = false } = {}) {
  const result = spawnSync(command, args, { env, cwd, encoding: 'utf8', stdio: capture ? 'pipe' : 'inherit', maxBuffer: 16 * 1024 * 1024 });
  if (capture && result.stderr) process.stderr.write(result.stderr);
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} 실패 (${result.status})`);
  return result.stdout;
}
