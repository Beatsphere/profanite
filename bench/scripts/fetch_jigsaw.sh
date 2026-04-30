#!/usr/bin/env bash
#
# Fetch a real-world English profanity corpus for the benchmark's `jigsaw`
# slot.
#
# Historical note: this gate was originally calibrated against
# google/jigsaw_toxicity_pred (CC0, ~220K Wikipedia comments). That
# dataset is no longer freely downloadable — it's been moved behind
# Kaggle auth / HF access gates. We fall back to
# `tdavidson/hate_speech_offensive` — 24.8K labeled tweets — as a
# proxy that keeps the gate meaningful on real-world text without
# requiring credentials.
#
# The schema conversion:
#   class=1 (offensive)  -> profane   (contains profanity)
#   class=2 (neither)    -> benign    (no profanity)
#   class=0 (hate)       -> dropped   (hate-without-profanity is out of scope)
#
# Usage: from repo root:
#   bash bench/scripts/fetch_jigsaw.sh [venv-python]
set -euo pipefail

WS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="${WS_ROOT}/bench/data/jigsaw.jsonl"
TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT

# Caller can pass a venv python; default to system python3.
PYTHON="${1:-python3}"

URL="https://huggingface.co/datasets/tdavidson/hate_speech_offensive/resolve/main/data/train-00000-of-00001.parquet"

echo "downloading hate_speech_offensive (proxy for Jigsaw, ~1.6 MB)..."
curl -fsSL "${URL}" -o "${TMP}/train.parquet"

"${PYTHON}" - "${TMP}/train.parquet" "${OUT}" <<'PY'
import json, sys
try:
    import pyarrow.parquet as pq
except ImportError:
    sys.stderr.write("this script needs pyarrow (`pip install pyarrow`)\n")
    sys.exit(1)

inp, out = sys.argv[1], sys.argv[2]
table = pq.read_table(inp)
rows = table.to_pylist()

keep_profane = 0
keep_benign = 0
dropped_hate = 0
with open(out, "w", encoding="utf-8") as f:
    f.write("# profanite jigsaw slice (proxy: tdavidson/hate_speech_offensive)\n")
    f.write("# class=1 -> profane ; class=2 -> benign ; class=0 (hate) dropped\n")
    for row in rows:
        cls = row.get("class")
        text = (row.get("tweet") or "").strip()
        if not text:
            continue
        if cls == 1:
            label = "profane"; keep_profane += 1
        elif cls == 2:
            label = "benign"; keep_benign += 1
        else:
            dropped_hate += 1
            continue
        rec = {"text": text, "label": label, "category": "realworld", "lang": "en"}
        f.write(json.dumps(rec, ensure_ascii=False) + "\n")

print(f"wrote {keep_profane} profane + {keep_benign} benign (dropped {dropped_hate} hate-only) to {out}")
PY
