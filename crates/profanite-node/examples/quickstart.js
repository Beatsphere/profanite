/**
 * Quickstart example — this file is the canonical Node usage snippet.
 *
 * The README pulls its JS code block directly from here via
 * `scripts/sync-readme.py`. If you change this example, the README
 * regenerates automatically; if this example breaks, CI fails.
 */

const { Profanite } = require('@beatsphere/profanite');

// Build a filter once, reuse for many inputs.
const filter = new Profanite({
  languages: ['en'],
  censorStyle: 'lengthPreserving',
});

// Detect.
console.assert(filter.containsProfanity('what the fuck') === true);
console.assert(filter.containsProfanity('have a nice day') === false);

// Censor. Default style masks each character with '*'.
console.assert(filter.censor('what the fuck') === 'what the ****');

// Locate. Each match carries spans + category + severity.
const hits = filter.find('oh fuck that');
console.assert(hits.length === 1);
console.assert(hits[0].start === 3 && hits[0].end === 7);

// Obfuscation-resistant matching covers leet, homoglyphs, repeats,
// zero-width chars, fullwidth, and bidi overrides.
console.assert(filter.containsProfanity('what the fuсk')); // Cyrillic 'с'
console.assert(filter.containsProfanity('fuuuuuuck'));
console.assert(filter.containsProfanity('ＦＵＣＫ'));

console.log('quickstart ok');
