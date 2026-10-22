import { cpSync, existsSync, mkdirSync, renameSync, rmSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { randomUUID } from 'node:crypto';

export function installBundle({ source, destination, backupDirectory, stop, activate }) {
  const parent = dirname(destination);
  mkdirSync(parent, { recursive: true });
  const lock = resolve(parent, '.amu-install.lock');
  mkdirSync(lock); // 동시 설치는 실패시킨다. 다른 설치의 잠금을 지우지 않는다.
  const staging = resolve(parent, `.amu-${randomUUID()}.app`);
  let backup;
  let installed = false;
  try {
    cpSync(source, staging, { recursive: true });
    stop();
    if (existsSync(destination)) {
      mkdirSync(backupDirectory, { recursive: true });
      backup = resolve(backupDirectory, `Amu-${Date.now()}-${randomUUID()}.app`);
      renameSync(destination, backup);
    }
    renameSync(staging, destination);
    installed = true;
    activate();
    return { status: 'installed', backup };
  } catch (error) {
    if (installed) {
      try {
        stop();
        rmSync(destination, { recursive: true });
        if (backup) renameSync(backup, destination);
      } catch (rollbackError) {
        throw new AggregateError([error, rollbackError], `설치와 복구를 완료하지 못했습니다. 보존된 백업: ${backup ?? '없음'}`);
      }
    } else if (backup) {
      renameSync(backup, destination);
    }
    throw error;
  } finally {
    rmSync(staging, { recursive: true, force: true });
    rmSync(lock, { recursive: true });
  }
}
