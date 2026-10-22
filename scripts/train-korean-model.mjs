import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const REVISION = '7628157f2ba54e0580b2c03e22f1a9f5b111d7e7';
const SOURCE_SHA256 = '0bd96ceae934d1089ece599b8904b43b0074df75e41a8e07e4b72264337b39f8';
const SOURCE_URL = `https://raw.githubusercontent.com/UniversalDependencies/UD_Korean-GSD/${REVISION}/ko_gsd-ud-train.conllu`;
const CACHE = resolve('research/upstream/korean-gsd/train.conllu');
const OUTPUT = resolve('models/korean');
const SYLLABLES = 11172;

const source = await loadSource();
const statistics = countSyllables(source.toString('utf8'));
writeModel(statistics);

async function loadSource() {
  let source;
  if (existsSync(CACHE)) source = readFileSync(CACHE);
  else {
    const response = await fetch(SOURCE_URL);
    if (!response.ok) throw new Error(`Corpus download failed: ${response.status}`);
    source = Buffer.from(await response.arrayBuffer());
  }
  const sha = createHash('sha256').update(source).digest('hex');
  if (sha !== SOURCE_SHA256) throw new Error(`Corpus checksum mismatch: ${sha}`);
  mkdirSync(resolve('research/upstream/korean-gsd'), { recursive: true });
  writeFileSync(CACHE, source);
  return source;
}

function countSyllables(source) {
  const unigrams = new Map();
  const bigrams = new Map();
  const outgoing = new Map();
  let tokens = 0;
  for (const line of source.split('\n')) {
    const columns = line.split('\t');
    if (!/^\d+$/.test(columns[0]) || !/^[가-힣]+$/.test(columns[1] ?? '')) continue;
    tokens += 1;
    const syllables = [...columns[1]].map(character => character.codePointAt(0) - 0xac00);
    for (let index = 0; index < syllables.length; index += 1) {
      const current = syllables[index];
      unigrams.set(current, (unigrams.get(current) ?? 0) + 1);
      if (index === 0) continue;
      const previous = syllables[index - 1];
      const pair = previous * SYLLABLES + current;
      bigrams.set(pair, (bigrams.get(pair) ?? 0) + 1);
      outgoing.set(previous, (outgoing.get(previous) ?? 0) + 1);
    }
  }
  return { unigrams, bigrams, outgoing, tokens };
}

function writeModel({ unigrams, bigrams, outgoing, tokens }) {
  const total = [...unigrams.values()].reduce((sum, count) => sum + count, 0);
  const unigramProbability = syllable => ((unigrams.get(syllable) ?? 0) + 0.1) / (total + 0.1 * SYLLABLES);
  const rows = ['# Amu Korean syllable statistics; UD Korean GSD train only; CC BY-SA 4.0'];
  for (const [syllable] of [...unigrams].sort(([left], [right]) => left - right)) {
    rows.push(`U\t${syllable}\t${Math.min(12, -Math.log(unigramProbability(syllable))).toFixed(6)}`);
  }
  for (const [pair, count] of [...bigrams].sort(([left], [right]) => left - right)) {
    const previous = Math.floor(pair / SYLLABLES);
    const current = pair % SYLLABLES;
    const probability = (count + 10 * unigramProbability(current)) / (outgoing.get(previous) + 10);
    rows.push(`B\t${pair}\t${Math.min(12, -Math.log(probability)).toFixed(6)}`);
  }
  const data = `${rows.join('\n')}\n`;
  mkdirSync(OUTPUT, { recursive: true });
  writeFileSync(resolve(OUTPUT, 'syllables.tsv'), data);
  writeFileSync(resolve(OUTPUT, 'manifest.json'), `${JSON.stringify({
    source: SOURCE_URL, revision: REVISION, sourceSha256: SOURCE_SHA256,
    license: 'CC-BY-SA-4.0', transform: 'Hangul-only FORM tokens; smoothed syllable unigrams and bigrams; no Latin terms or word lookup',
    tokens, syllables: total, unigramEntries: unigrams.size, bigramEntries: bigrams.size,
    modelSha256: createHash('sha256').update(data).digest('hex'),
  }, null, 2)}\n`);
  console.log(`Generated Korean statistics from ${tokens} tokens: ${unigrams.size} syllables, ${bigrams.size} pairs`);
}
