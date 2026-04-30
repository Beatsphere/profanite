"""Smoke test for the Python binding.

Mirrors crates/profanite-node/test/smoke.js so the two language bindings
stay behaviorally aligned. Run with `pytest -q` or directly:
    python3 -m pytest crates/profanite-py/tests/test_smoke.py
"""

from profanite import Match, Profanite


def test_default_constructor():
    p = Profanite()
    assert p.contains_profanity("hello world") is False
    assert p.contains_profanity("what the fuck") is True


def test_find_returns_correct_spans():
    p = Profanite()
    hits = p.find("oh fuck that")
    assert len(hits) == 1
    assert isinstance(hits[0], Match)
    assert hits[0].start == 3
    assert hits[0].end == 7
    assert hits[0].category == "strong"
    assert hits[0].severity == 3


def test_censor_masks_the_full_match():
    p = Profanite()
    assert p.censor("oh fuck that") == "oh **** that"


def test_obfuscation_homoglyph():
    p = Profanite()
    # Cyrillic 'с' (U+0441) in place of ASCII 'c'
    assert p.contains_profanity("what the fuсk") is True


def test_concat_bypass_caught_by_tier3_strict():
    p = Profanite()
    assert p.contains_profanity("Hemoglomotherfuckerbin") is True


def test_scunthorpe_protection():
    p = Profanite()
    assert p.contains_profanity("classroom assignment") is False
    assert p.contains_profanity("passing the ball") is False
    assert p.contains_profanity("my hemoglobin levels") is False


def test_custom_mask_char_and_first_last_style():
    p = Profanite({"mask_char": "#", "censor_style": "first_last"})
    assert p.censor("oh fuck that") == "oh f##k that"


def test_multi_language_bundle():
    p = Profanite({"languages": ["en", "es", "fr", "de"]})
    assert p.contains_profanity("tu eres cabrón") is True
    assert p.contains_profanity("putain de merde") is True
    assert p.contains_profanity("du Arschloch") is True
    assert p.contains_profanity("hello bonjour guten tag") is False


def test_add_words_extends_wordlist():
    p = Profanite({
        "add_words": [
            {"word": "meanieword", "category": "mild", "severity": 1, "strict": False},
        ],
    })
    assert p.contains_profanity("that was a meanieword") is True


def test_allowlist_suppresses_overlapping_matches():
    p = Profanite({
        "without_bundled": True,
        "add_words": [{"word": "ass", "category": "mild", "severity": 1, "strict": False}],
        "match_mode": "substring",
        "allowlist": ["ass-occurence-here"],
    })
    assert p.contains_profanity("there is an ass-occurence-here") is False
    assert p.contains_profanity("just ass alone") is True


def test_invalid_language_raises():
    import pytest
    with pytest.raises(ValueError, match="unknown language code"):
        Profanite({"languages": ["klingon"]})


def test_match_repr_is_readable():
    p = Profanite()
    hits = p.find("oh fuck that")
    repr_str = repr(hits[0])
    assert "start=3" in repr_str
    assert "category='strong'" in repr_str
