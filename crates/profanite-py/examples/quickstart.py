"""Quickstart example — canonical Python usage snippet.

The README pulls this file's content verbatim via
`scripts/sync-readme.py`. If you change this example, the README
regenerates automatically; if this example breaks, CI fails.
"""

from profanite import Profanite

# Build once, reuse for many inputs.
p = Profanite({
    "languages": ["en"],
    "censor_style": "length_preserving",
})

# Detect.
assert p.contains_profanity("what the fuck") is True
assert p.contains_profanity("have a nice day") is False

# Censor. Default style masks each character with '*'.
assert p.censor("what the fuck") == "what the ****"

# Locate. Each match carries spans + category + severity.
hits = p.find("oh fuck that")
assert len(hits) == 1
assert hits[0].start == 3 and hits[0].end == 7

# Obfuscation-resistant matching covers leet, homoglyphs, repeats,
# zero-width chars, fullwidth, and bidi overrides.
assert p.contains_profanity("what the fuсk")  # Cyrillic 'с'
assert p.contains_profanity("fuuuuuuck")
assert p.contains_profanity("ＦＵＣＫ")

print("quickstart ok")
