#!/bin/bash

set -euo pipefail

files=(infra/nightly-resources/*.js)

if command -v prettier >/dev/null 2>&1; then
    exec prettier "${files[@]}" --check
fi

if [ -n "${CI:-}" ] || [ -n "${FORCE_NPX_PRETTIER:-}" ]; then
    exec npx prettier "${files[@]}" --check
fi

echo "Skipping prettier check: no local prettier found and CI is not set." >&2
