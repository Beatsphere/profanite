/**
 * TypeScript declarations for profanite. Kept in lockstep with
 * `src/lib.rs`; any JS-facing change there must be mirrored here.
 */

/** Two-letter language code for a bundled wordlist. */
export type Language = "en" | "es" | "hi" | "fr" | "de";

/** How aggressively to normalize the input before matching. */
export type Normalization = "none" | "basic" | "aggressive";

/** Does a match require surrounding non-word characters? */
export type MatchMode = "wordBoundary" | "substring";

/** Strategy for rendering masked profanity. */
export type CensorStyle =
  | "lengthPreserving"
  | "firstLast"
  | "fullMask"
  | "grawlix";

/** Semantic bucket attached to a match. */
export type Category = "mild" | "strong" | "sexual" | "slur" | "slang";

export interface CustomWord {
  word: string;
  category: Category;
  /** 1..=3 severity band (1 = mild, 3 = most severe). */
  severity: number;
  /**
   * If true, the matcher ignores word-boundary checks for this entry —
   * useful for long, unambiguous compounds like "motherfucker" where
   * any substring appearance is profane.
   */
  strict: boolean;
}

export interface ProfaniteOptions {
  /** Defaults to ["en"]. */
  languages?: Language[];
  /** Defaults to "basic". */
  normalization?: Normalization;
  /** Defaults to "wordBoundary". */
  matchMode?: MatchMode;
  /** Defaults to "lengthPreserving". */
  censorStyle?: CensorStyle;
  /** Single character used for masking. Defaults to "*". */
  maskChar?: string;
  /** Additional wordlist entries, merged with the bundled defaults. */
  addWords?: CustomWord[];
  /** Words to remove from the bundled list (case-insensitive). */
  removeWords?: string[];
  /**
   * Substrings where any overlapping match is suppressed
   * (case-insensitive). Primary escape hatch for the Scunthorpe problem.
   */
  allowlist?: string[];
  /** Skip bundled languages entirely; caller must populate via `addWords`. */
  withoutBundled?: boolean;
}

export interface ProfanityMatch {
  /** Stable identifier of the matched word within this Profanite instance. */
  wordId: number;
  /** Byte offset in the original input where the match starts. */
  start: number;
  /** Byte offset (exclusive) where the match ends. */
  end: number;
  /** Byte offset of the match in the normalized form of the input. */
  normalizedStart: number;
  /** Exclusive byte offset in the normalized form. */
  normalizedEnd: number;
  category: Category;
  severity: number;
}

/**
 * Profanity filter. Construct once and reuse — the aho-corasick
 * automaton is built at construction time.
 *
 * @example
 * ```js
 * const { Profanite } = require('profanite');
 * const p = new Profanite({ languages: ['en'] });
 * p.containsProfanity('what the fuck'); // -> true
 * p.censor('what the fuck');             // -> 'what the ****'
 * p.find('what the fuck');               // -> [{ wordId, start, end, ... }]
 * ```
 */
export class Profanite {
  constructor(options?: ProfaniteOptions);
  containsProfanity(text: string): boolean;
  censor(text: string): string;
  find(text: string): ProfanityMatch[];
}
