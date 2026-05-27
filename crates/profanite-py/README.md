# profanite (Python)

Python bindings for [profanite](https://github.com/Beatsphere/profanite) — a lightweight, obfuscation-resistant profanity filter.

## Install

```bash
pip install profanite
```

Prebuilt wheels for Linux (manylinux + musllinux, x86_64 + aarch64), macOS (x86_64 + arm64), and Windows x64. Python 3.8+ via the stable `abi3` ABI.

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
