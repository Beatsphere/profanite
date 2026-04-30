# profanite (Python)

Python bindings for [profanite](https://github.com/Beatsphere/profanite) — a lightweight, obfuscation-resistant profanity filter.

## Install

Until v0.1 ships to PyPI, build locally:

```bash
pip install maturin
cd crates/profanite-py
maturin develop --release
```

## Usage

```python
from profanite import Profanite

p = Profanite({"languages": ["en"]})

p.contains_profanity("what the fuck")   # True
p.censor("what the fuck")               # "what the ****"
hits = p.find("oh fuck that")
hits[0].start, hits[0].end               # (3, 7)
```

See the top-level README for the full configuration reference.
