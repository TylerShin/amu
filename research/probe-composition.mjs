import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const REVISION = '23d2cd9d10d516d523d60c562dbcc7ea08139a09';
const SOURCE = resolve(ROOT, 'research/upstream/libhangul');
const BUILD = resolve(ROOT, 'research/build');
const EXECUTABLE = resolve(BUILD, 'trace-libhangul');
const EVIDENCE = resolve(ROOT, 'research/composition-evidence.json');
const ONSETS = ['r', 'R', 's', 'e', 'E', 'f', 'a', 'q', 'Q', 't', 'T', 'd', 'w', 'W', 'c', 'z', 'x', 'v', 'g'];
const VOWELS = ['k', 'o', 'i', 'O', 'j', 'p', 'u', 'P', 'h', 'hk', 'ho', 'hl', 'y', 'n', 'nj', 'np', 'nl', 'b', 'm', 'ml', 'l'];
const CODAS = ['', 'r', 'R', 'rt', 's', 'sw', 'sg', 'e', 'f', 'fr', 'fa', 'fq', 'ft', 'fx', 'fv', 'fg', 'a', 'q', 'qt', 't', 'T', 'd', 'w', 'c', 'z', 'x', 'v', 'g'];
const CASES = [
  { name: 'compound coda crosses a syllable boundary', keys: 'rkqtk<<<', shown: ['ㄱ', '가', '갑', '값', '갑사', '갑ㅅ', '갑', ''] },
  { name: 'compound vowel deletes one component', keys: 'rhk<<<', shown: ['ㄱ', '고', '과', '고', 'ㄱ', ''] },
  { name: 'simple coda crosses a syllable boundary', keys: 'rksk<', shown: ['ㄱ', '가', '간', '가나', '가ㄴ'] },
  { name: 'a committed syllable is deleted as host text', keys: 'rk|<', shown: ['ㄱ', '가', '가', ''] },
  { name: 'shifted onset and vowel', keys: 'RO<<', shown: ['ㄲ', '꺠', 'ㄲ', ''] },
  { name: 'punctuation and space commit exactly once', keys: 'gks, ', shown: ['ㅎ', '하', '한', '한,', '한, '] },
];

main();

function main() {
  prepareReferenceSource();
  compileReferenceProbe();
  const syllables = createSyllableCases();
  const traces = runTraces(CASES.map(({ keys }) => keys));
  for (const [index, trace] of traces.entries()) {
    assert.deepEqual(trace.map(({ shown }) => shown), CASES[index].shown, CASES[index].name);
  }
  const composed = runTraces(syllables.map(({ keys }) => `${keys}|`));
  for (const [index, trace] of composed.entries()) {
    assert.equal(trace.at(-1).shown, syllables[index].text, syllables[index].keys);
    assert.equal(trace.at(-1).preedit, '', 'flush must empty the preedit');
  }
  const erased = runTraces(syllables.map(({ keys }) => `${keys}${'<'.repeat(keys.length)}|`));
  for (const [index, trace] of erased.entries()) {
    assert.equal(trace.at(-1).shown, '', `erase ${syllables[index].keys}`);
  }

  const evidence = {
    executedAt: new Date().toISOString(),
    reference: { repository: 'https://github.com/libhangul/libhangul', revision: REVISION, keyboard: '2', options: 'library defaults' },
    environment: { node: process.version, platform: process.platform, architecture: process.arch, compiler: run('clang', ['--version']).split('\n')[0] },
    status: 'passed',
    checks: { composedModernSyllables: composed.length, erasedModernSyllables: erased.length, curatedTraces: traces.length },
    limitations: ['Reference-library checks only; no Amu composer exists yet.', 'The host model supports only these Korean/ASCII traces; it is not a grapheme-aware editor.', 'No real OS, browser, focus, selection, or IME callback validation.'],
    traces: traces.map((steps, index) => ({ name: CASES[index].name, keys: CASES[index].keys, steps })),
  };
  writeFileSync(EVIDENCE, `${JSON.stringify(evidence, null, 2)}\n`);
  console.log(JSON.stringify({ status: evidence.status, ...evidence.checks, evidence: EVIDENCE }, null, 2));
}

function prepareReferenceSource() {
  if (!existsSync(SOURCE)) {
    mkdirSync(dirname(SOURCE), { recursive: true });
    run('git', ['clone', '--no-checkout', '--depth', '1', 'https://github.com/libhangul/libhangul.git', SOURCE]);
    run('git', ['-C', SOURCE, 'fetch', '--depth', '1', 'origin', REVISION]);
    run('git', ['-C', SOURCE, 'checkout', '--detach', REVISION]);
  }
  assert.equal(run('git', ['-C', SOURCE, 'rev-parse', 'HEAD']).trim(), REVISION, 'Reference revision differs; existing source is left untouched.');
  run('git', ['-C', SOURCE, 'diff', '--exit-code', REVISION, '--', 'hangul']);
}

function compileReferenceProbe() {
  mkdirSync(BUILD, { recursive: true });
  const hangul = resolve(SOURCE, 'hangul');
  run('clang', [
    '-std=c11', '-O2', '-Wall', '-Wextra', '-DENABLE_EXTERNAL_KEYBOARDS=0',
    '-I', hangul,
    resolve(ROOT, 'research/trace-libhangul.c'),
    ...['hangulctype.c', 'hangulinputcontext.c', 'hangulkeyboard.c'].map((file) => resolve(hangul, file)),
    '-o', EXECUTABLE,
  ]);
}

function createSyllableCases() {
  const cases = [];
  for (const [onset, onsetKeys] of ONSETS.entries()) {
    for (const [vowel, vowelKeys] of VOWELS.entries()) {
      for (const [coda, codaKeys] of CODAS.entries()) {
        cases.push({ keys: onsetKeys + vowelKeys + codaKeys, text: String.fromCodePoint(0xAC00 + (onset * 21 + vowel) * 28 + coda) });
      }
    }
  }
  return cases;
}

function runTraces(inputs) {
  const lines = run(EXECUTABLE, [], `${inputs.join('\n')}\n`).trimEnd().split('\n');
  assert.equal(lines.length, inputs.length, 'Every input must produce one trace');
  return lines.map((line, index) => simulateHost(inputs[index], JSON.parse(line)));
}

function simulateHost(keys, steps) {
  let committed = '';
  assert.equal(steps.length, keys.length, 'Every operation must produce one step');
  return steps.map((step, index) => {
    const commit = String.fromCodePoint(...step.commit);
    const preedit = String.fromCodePoint(...step.preedit);
    committed += commit;
    if (!step.consumed) {
      // This fixture host handles modern Hangul and ASCII only.
      committed = keys[index] === '<' ? Array.from(committed).slice(0, -1).join('') : committed + keys[index];
    }
    return { operation: keys[index], consumed: step.consumed, commit, preedit, shown: committed + preedit };
  });
}

function run(command, args, input) {
  const result = spawnSync(command, args, { cwd: ROOT, input, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} failed (${result.status}): ${result.stderr || result.stdout}`);
  return result.stdout;
}
