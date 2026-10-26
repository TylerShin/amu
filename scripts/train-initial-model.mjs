import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

const SOURCES = [
  {
    name: 'korean-gsd', language: 'Korean',
    revision: '7628157f2ba54e0580b2c03e22f1a9f5b111d7e7',
    url: 'https://raw.githubusercontent.com/UniversalDependencies/UD_Korean-GSD/7628157f2ba54e0580b2c03e22f1a9f5b111d7e7/ko_gsd-ud-train.conllu',
    sha256: '0bd96ceae934d1089ece599b8904b43b0074df75e41a8e07e4b72264337b39f8',
  },
  {
    name: 'english-ewt', language: 'English',
    revision: '4a4d77f599ea53cc405f85d0cec4b2f14f81d42b',
    url: 'https://raw.githubusercontent.com/UniversalDependencies/UD_English-EWT/4a4d77f599ea53cc405f85d0cec4b2f14f81d42b/en_ewt-ud-train.conllu',
    sha256: 'd68e06122a702464c613076523d56740f047e5bbe89dd90ec32737e04d952143',
  },
];
const ONSET_KEYS = 'rRseEfaqQtTdwWczxvg';
const PAIRS = 26 * 26;
const SMOOTHING = 0.5;
const OUTPUT = resolve('models/initials');

const statistics = [];
for (const source of SOURCES) {
  const corpus = await loadSource(source);
  statistics.push(countPairs(corpus, source.language));
}
writeModel(statistics);

async function loadSource(source) {
  const cache = resolve(`research/upstream/${source.name}/train.conllu`);
  let data;
  if (existsSync(cache)) data = readFileSync(cache);
  else {
    const response = await fetch(source.url);
    if (!response.ok) throw new Error(`Corpus download failed: ${response.status}`);
    data = Buffer.from(await response.arrayBuffer());
  }
  if (createHash('sha256').update(data).digest('hex') !== source.sha256) {
    throw new Error(`Corpus checksum mismatch: ${source.name}`);
  }
  mkdirSync(dirname(cache), { recursive: true });
  writeFileSync(cache, data);
  return data.toString('utf8');
}

function countPairs(corpus, language) {
  const counts = Array(PAIRS).fill(0);
  const forms = new Set();
  let tokens = 0;
  for (const line of corpus.split('\n')) {
    const [id, form] = line.split('\t');
    if (!/^\d+$/.test(id)) continue;
    let keys;
    if (language === 'Korean') {
      if (!/^[가-힣]{2,}$/.test(form ?? '')) continue;
      keys = [...form].map(character => ONSET_KEYS[Math.floor((character.codePointAt(0) - 0xac00) / 588)]).join('');
    } else {
      // Match the class being disambiguated: short consonant-only Latin tokens.
      // Ordinary words with a vowel (including y) have separate spelling evidence.
      if (!/^[a-zA-Z]{2,6}$/.test(form ?? '') || /[aeiouy]/i.test(form)) continue;
      keys = form.toLowerCase();
    }
    tokens += 1;
    // Count distinct written forms once. Repeated names or boilerplate in a
    // small treebank must not dominate the evidence for an entire pair.
    const identity = language === 'English' ? form.toLowerCase() : form;
    if (forms.has(identity)) continue;
    forms.add(identity);
    for (let index = 1; index < keys.length; index += 1) {
      const pair = keys.slice(index - 1, index + 1);
      if (!/^[a-z]{2}$/.test(pair)) continue; // Shifted onsets are not unshifted initials.
      counts[(pair.charCodeAt(0) - 97) * 26 + pair.charCodeAt(1) - 97] += 1;
    }
  }
  return { counts, tokens, forms: forms.size, pairs: counts.reduce((sum, count) => sum + count, 0) };
}

function writeModel(statistics) {
  const rows = ['# Amu onset/Latin pair statistics; UD training partitions only; CC BY-SA 4.0'];
  for (let pair = 0; pair < PAIRS; pair += 1) {
    const costs = statistics.map(({ counts, pairs }) => -Math.log((counts[pair] + SMOOTHING) / (pairs + SMOOTHING * PAIRS)));
    rows.push(`${pair}\t${costs.map(cost => cost.toFixed(6)).join('\t')}`);
  }
  const data = `${rows.join('\n')}\n`;
  mkdirSync(OUTPUT, { recursive: true });
  writeFileSync(resolve(OUTPUT, 'pairs.tsv'), data);
  writeFileSync(resolve(OUTPUT, 'manifest.json'), `${JSON.stringify({
    sources: SOURCES.map((source, index) => ({ ...source, tokens: statistics[index].tokens, forms: statistics[index].forms, pairs: statistics[index].pairs })),
    license: 'CC-BY-SA-4.0', smoothing: SMOOTHING,
    transform: 'Distinct FORM types: adjacent unshifted onsets of Hangul-only forms vs adjacent letters of 2-6 letter Latin forms without a/e/i/o/u/y (case folded); no word lookup or labelled expression list',
    entries: PAIRS, modelSha256: createHash('sha256').update(data).digest('hex'),
  }, null, 2)}\n`);
  console.log(`Generated initial evidence: ${PAIRS} pairs, Korean=${statistics[0].pairs}, Latin=${statistics[1].pairs}`);
}
