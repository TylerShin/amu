import { test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { installBundle } from '../install-bundle.mjs';

test('실행 확인에 성공해야 설치 성공을 반환하고 이전 앱을 백업한다', (context) => {
  const files = fixture(context);
  const effects = [];
  const result = installBundle({ ...files, stop: () => effects.push('stop'), activate: () => {
    assert.equal(contents(files.destination), 'new');
    effects.push('verified');
  } });
  assert.equal(result.status, 'installed');
  assert.equal(contents(result.backup), 'old');
  assert.equal(contents(files.destination), 'new');
  assert.deepEqual(effects, ['stop', 'verified']);
  assertClean(files);
});

test('입력기가 종료를 거부하면 설치본과 실행 확인 경로를 건드리지 않는다', (context) => {
  const files = fixture(context);
  assert.throws(() => installBundle({ ...files, stop: () => { throw new Error('busy'); }, activate: () => assert.fail('must not activate') }), /busy/);
  assert.equal(contents(files.destination), 'old');
  assert.equal(existsSync(files.backupDirectory), false);
  assertClean(files);
});

test('버전 확인에 실패하면 새 프로세스를 멈춘 뒤 이전 앱을 복구한다', (context) => {
  const files = fixture(context);
  let stops = 0;
  assert.throws(() => installBundle({ ...files, stop: () => { stops++; }, activate: () => { throw new Error('wrong version'); } }), /wrong version/);
  assert.equal(stops, 2);
  assert.equal(contents(files.destination), 'old');
  assertClean(files);
});

test('복구 중 종료를 거부하면 실행 중인 앱과 백업을 모두 보존한다', (context) => {
  const files = fixture(context);
  let stops = 0;
  assert.throws(() => installBundle({ ...files, stop: () => { if (++stops === 2) throw new Error('busy'); }, activate: () => { throw new Error('no reply'); } }), AggregateError);
  assert.equal(contents(files.destination), 'new');
  const backups = readdirSync(files.backupDirectory);
  assert.equal(backups.length, 1);
  assert.equal(contents(resolve(files.backupDirectory, backups[0])), 'old');
  assertClean(files);
});

test('첫 설치의 실행 실패는 잘못 설치한 앱을 제거한다', (context) => {
  const files = fixture(context);
  rmSync(files.destination, { recursive: true });
  assert.throws(() => installBundle({ ...files, stop: () => {}, activate: () => { throw new Error('launch failed'); } }), /launch failed/);
  assert.equal(existsSync(files.destination), false);
  assertClean(files);
});

test('다른 설치가 보유한 잠금은 제거하거나 덮어쓰지 않는다', (context) => {
  const files = fixture(context);
  const lock = resolve(files.parent, '.amu-install.lock');
  mkdirSync(lock);
  assert.throws(() => installBundle({ ...files, stop: () => assert.fail(), activate: () => assert.fail() }), { code: 'EEXIST' });
  assert.equal(contents(files.destination), 'old');
  assert.ok(existsSync(lock));
});

function fixture(context) {
  const root = mkdtempSync(resolve(tmpdir(), 'amu-install-test-'));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const parent = resolve(root, 'Input Methods');
  const source = resolve(root, 'source.app');
  const destination = resolve(parent, 'Amu.app');
  for (const [path, value] of [[source, 'new'], [destination, 'old']]) {
    mkdirSync(path, { recursive: true });
    writeFileSync(resolve(path, 'binary'), value);
  }
  return { source, destination, parent, backupDirectory: resolve(root, 'backups') };
}

function contents(bundle) { return readFileSync(resolve(bundle, 'binary'), 'utf8'); }
function assertClean(files) { assert.deepEqual(readdirSync(files.parent).filter((name) => name.startsWith('.amu-')), []); }
