#!/usr/bin/env bash

set -euo pipefail

if [[ "$#" -ne 3 ]]; then
  echo "usage: stage-npm-package.sh <version> <artifacts-dir> <output-dir>" >&2
  exit 2
fi

version="$1"
artifacts_dir="$2"
output_dir="$3"

if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-+][0-9A-Za-z.-]+)?$ ]]; then
  echo "invalid package version: $version" >&2
  exit 2
fi
if [[ ! -d "$artifacts_dir" ]]; then
  echo "artifact directory not found: $artifacts_dir" >&2
  exit 2
fi
if [[ -e "$output_dir" ]]; then
  echo "output path already exists: $output_dir" >&2
  exit 2
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repository_root="$(cd "$script_dir/.." && pwd -P)"
package_source="$repository_root/npm"
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT

stage_root="$scratch/package"
mkdir -p "$stage_root/bin" "$stage_root/lib" "$stage_root/tests"
cp "$package_source/package.json" "$stage_root/package.json"
cp "$package_source/bin/arthur-skills.js" "$stage_root/bin/arthur-skills.js"
cp "$package_source/lib/target.js" "$stage_root/lib/target.js"
cp "$package_source/tests/launcher.test.js" "$stage_root/tests/launcher.test.js"
cp "$package_source/tests/target.test.js" "$stage_root/tests/target.test.js"
cp "$repository_root/LICENSE" "$stage_root/LICENSE"
cp "$repository_root/LICENSE-APACHE-2.0" "$stage_root/LICENSE-APACHE-2.0"
cp "$repository_root/README.md" "$stage_root/README.md"
cp "$repository_root/THIRD_PARTY.md" "$stage_root/THIRD_PARTY.md"

jq --arg version "$version" '.version = $version | del(.private)' \
  "$stage_root/package.json" > "$scratch/package.json"
mv "$scratch/package.json" "$stage_root/package.json"

targets=(
  x86_64-unknown-linux-musl
  aarch64-unknown-linux-musl
  x86_64-apple-darwin
  aarch64-apple-darwin
  x86_64-pc-windows-msvc
)

for target in "${targets[@]}"; do
  extraction_root="$scratch/extract-$target"
  mkdir -p "$extraction_root"

  if [[ "$target" == "x86_64-pc-windows-msvc" ]]; then
    archive="$artifacts_dir/arthur-skills-$target.zip"
    binary_name="arthur-skills.exe"
    if [[ ! -f "$archive" ]]; then
      echo "native archive not found: $archive" >&2
      exit 1
    fi
    unzip -q "$archive" -d "$extraction_root"
  else
    archive="$artifacts_dir/arthur-skills-$target.tar.xz"
    binary_name="arthur-skills"
    if [[ ! -f "$archive" ]]; then
      echo "native archive not found: $archive" >&2
      exit 1
    fi
    tar -xJf "$archive" -C "$extraction_root"
  fi

  mapfile -t binaries < <(
    find "$extraction_root" -type f -name "$binary_name" -print
  )
  if [[ "${#binaries[@]}" -ne 1 ]]; then
    echo "expected one $binary_name in $archive, found ${#binaries[@]}" >&2
    exit 1
  fi

  target_root="$stage_root/vendor/$target"
  mkdir -p "$target_root"
  cp "${binaries[0]}" "$target_root/$binary_name"
  if [[ "$target" != "x86_64-pc-windows-msvc" ]]; then
    chmod 755 "$target_root/$binary_name"
  fi
done

jq -e \
  --arg version "$version" \
  '.name == "@arthjean/skills" and .version == $version and
   .private == null and .bin["arthur-skills"] == "bin/arthur-skills.js"' \
  "$stage_root/package.json" >/dev/null

mkdir -p "$(dirname "$output_dir")"
mv "$stage_root" "$output_dir"
