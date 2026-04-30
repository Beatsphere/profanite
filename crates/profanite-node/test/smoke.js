/**
 * Smoke test for the profanite Node binding.
 *
 * Designed to fail loudly with a descriptive message so CI can surface
 * regressions. Uses only Node's built-in `assert` — no test framework
 * dependency.
 */

const assert = require('node:assert/strict');
const { Profanite } = require('..');

function section(label, fn) {
  process.stdout.write(`  ${label} ... `);
  try {
    fn();
    process.stdout.write('ok\n');
  } catch (e) {
    process.stdout.write('FAIL\n');
    throw e;
  }
}

console.log('profanite node smoke test');

section('default constructor', () => {
  const p = new Profanite();
  assert.equal(p.containsProfanity('hello world'), false);
  assert.equal(p.containsProfanity('what the fuck'), true);
});

section('find returns correct spans', () => {
  const p = new Profanite();
  const hits = p.find('oh fuck that');
  assert.equal(hits.length, 1);
  assert.equal(hits[0].start, 3);
  assert.equal(hits[0].end, 7);
  assert.equal(hits[0].category, 'strong');
  assert.equal(hits[0].severity, 3);
});

section('censor masks the full match', () => {
  const p = new Profanite();
  assert.equal(p.censor('oh fuck that'), 'oh **** that');
});

section('obfuscation-resistant matching (homoglyph)', () => {
  const p = new Profanite();
  // Cyrillic 'с' (U+0441) in place of ASCII 'c'
  assert.equal(p.containsProfanity('what the fuсk'), true);
});

section('concat bypass caught by tier-3 strict', () => {
  const p = new Profanite();
  assert.equal(p.containsProfanity('Hemoglomotherfuckerbin'), true);
});

section('Scunthorpe protection (word-boundary default)', () => {
  const p = new Profanite();
  assert.equal(p.containsProfanity('classroom assignment'), false);
  assert.equal(p.containsProfanity('passing the ball'), false);
  assert.equal(p.containsProfanity('my hemoglobin levels'), false);
});

section('custom options: maskChar + firstLast style', () => {
  const p = new Profanite({ maskChar: '#', censorStyle: 'firstLast' });
  assert.equal(p.censor('oh fuck that'), 'oh f##k that');
});

section('multi-language bundle', () => {
  const p = new Profanite({ languages: ['en', 'es', 'fr', 'de'] });
  assert.equal(p.containsProfanity('tu eres cabrón'), true);
  assert.equal(p.containsProfanity('putain de merde'), true);
  assert.equal(p.containsProfanity('du Arschloch'), true);
  assert.equal(p.containsProfanity('hello bonjour guten tag'), false);
});

section('addWords extends the wordlist', () => {
  const p = new Profanite({
    addWords: [
      { word: 'meanieword', category: 'mild', severity: 1, strict: false },
    ],
  });
  assert.equal(p.containsProfanity('that was a meanieword'), true);
});

section('allowlist suppresses overlapping matches', () => {
  const p = new Profanite({
    withoutBundled: true,
    addWords: [{ word: 'ass', category: 'mild', severity: 1, strict: false }],
    matchMode: 'substring',
    allowlist: ['ass-occurence-here'],
  });
  // "ass" appears inside the allowlisted token, suppressed.
  assert.equal(p.containsProfanity('there is an ass-occurence-here'), false);
  // Standalone still fires.
  assert.equal(p.containsProfanity('just ass alone'), true);
});

section('invalid language rejects with a descriptive error', () => {
  assert.throws(
    () => new Profanite({ languages: ['klingon'] }),
    /unknown language code: klingon/
  );
});

console.log('\nall smoke tests passed');

// Also execute the README-exported quickstart example to keep the docs
// honest. We rewrite the sole published-name require() on the fly so it
// resolves to this in-tree package.
section('quickstart example from examples/quickstart.js runs', () => {
  const fs = require('node:fs');
  const path = require('node:path');
  const Module = require('node:module');

  const examplePath = path.join(__dirname, '..', 'examples', 'quickstart.js');
  const original = fs.readFileSync(examplePath, 'utf8');
  const patched = original.replace(
    "require('@beatsphere/profanite')",
    "require('..')"
  );
  // Evaluate the patched source in a fresh module scope so it really
  // executes top-to-bottom the same way a consumer would run it.
  const mod = new Module(examplePath);
  mod.filename = examplePath;
  mod.paths = Module._nodeModulePaths(path.dirname(examplePath));
  mod._compile(patched, examplePath);
});

console.log('quickstart example passed');
