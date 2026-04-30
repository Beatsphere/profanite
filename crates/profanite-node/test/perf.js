/** Quick FFI-overhead sanity check. Not a gate, just a data point. */

const { Profanite } = require('..');

const p = new Profanite();
const CORPUS = [
  'The quick brown fox jumps over the lazy dog.',
  'She sells seashells by the seashore, and nobody said a damn word about it.',
  'In a village of La Mancha, the name of which I have no desire to call to mind.',
  'It was the best of times, it was the worst of times.',
  'He muttered something about the fucking weather and walked off.',
  'All happy families are alike; each unhappy family is unhappy in its own way.',
].join(' ');

const ITERATIONS = 100_000;

// Warm up
for (let i = 0; i < 1000; i++) p.containsProfanity(CORPUS);

console.log(`corpus size: ${CORPUS.length} bytes`);

for (const method of ['containsProfanity', 'find', 'censor']) {
  const start = process.hrtime.bigint();
  for (let i = 0; i < ITERATIONS; i++) {
    p[method](CORPUS);
  }
  const elapsedNs = Number(process.hrtime.bigint() - start);
  const perCallUs = elapsedNs / ITERATIONS / 1000;
  const mbPerSec = (CORPUS.length * ITERATIONS / (elapsedNs / 1e9)) / 1e6;
  console.log(`  ${method.padEnd(20)} ${perCallUs.toFixed(2)} µs/call · ${mbPerSec.toFixed(1)} MB/s`);
}
