#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

required=(
  README.md
  PRODUCT-SCOPE.md
  DOMAIN-MODEL.md
  ARCHITECTURE.md
  TECHNICAL.md
  DESIGN.md
  PLAN.md
  AGENTS.md
  docs/specs/workspace.schema.json
  docs/security/THREAT-MODEL.md
  prompts/00-MASTER-AI-DEVELOPMENT-PROMPT.md
  .agents/skills/flaredeck-implementation/SKILL.md
)

for file in "${required[@]}"; do
  if [[ ! -s "$ROOT/$file" ]]; then
    echo "Missing or empty required file: $file" >&2
    exit 1
  fi
done

python3 - <<'PY' "$ROOT/docs/specs/workspace.schema.json"
import json, sys
with open(sys.argv[1], encoding="utf-8") as f:
    json.load(f)
print("JSON schema: valid JSON")
PY

for skill in "$ROOT"/.agents/skills/*/SKILL.md; do
  parent="$(basename "$(dirname "$skill")")"
  name="$(awk '/^name: / {print $2; exit}' "$skill")"
  if [[ "$name" != "$parent" ]]; then
    echo "Skill name '$name' does not match parent directory '$parent': $skill" >&2
    exit 1
  fi
  if ! grep -q '^description: ' "$skill"; then
    echo "Skill missing description: $skill" >&2
    exit 1
  fi
done

echo "Agent Skills: valid basic structure"

echo "Markdown files: $(find "$ROOT" -type f -name '*.md' | wc -l)"
echo "Package validation passed."
