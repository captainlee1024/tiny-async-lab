#!/usr/bin/env bash

set -euo pipefail

output_dir="$(mktemp -d)"
trap 'rm -rf "$output_dir"' EXIT

if [[ -z "${PUPPETEER_EXECUTABLE_PATH:-}" ]]; then
    for browser in google-chrome-stable google-chrome chromium chromium-browser; do
        if browser_path="$(command -v "$browser")"; then
            export PUPPETEER_EXECUTABLE_PATH="$browser_path"
            break
        fi
    done
fi

if [[ -z "${PUPPETEER_EXECUTABLE_PATH:-}" ]]; then
    echo "Chrome or Chromium is required to render Mermaid diagrams." >&2
    exit 1
fi

mapfile -d '' -t markdown_files < <(
    rg --files --hidden --null --glob '*.md' --glob '!.git/**'
)

rendered_files=0

for markdown_file in "${markdown_files[@]}"; do
    if ! rg --quiet '^[[:space:]]*```mermaid[[:space:]]*$' "$markdown_file"; then
        continue
    fi

    output_file="$output_dir/${markdown_file//\//_}"
    mmdc --input "$markdown_file" --output "$output_file"
    rendered_files=$((rendered_files + 1))
done

echo "Rendered Mermaid diagrams in $rendered_files Markdown file(s)."
