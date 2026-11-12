#!/usr/bin/env bash
# Download a release archive, verify its SHA-256 digest, and install a binary.
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: install_pinned_release.sh --url URL --sha256 HEX --bin NAME [--archive-member PATH] [--strip-components N] [--dest DIR]
USAGE
}

url=""
sha256=""
bin_name=""
archive_member=""
strip_components=0
dest="${HOME}/.local/bin"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --url) url="${2:-}"; shift 2 ;;
    --sha256) sha256="${2:-}"; shift 2 ;;
    --bin) bin_name="${2:-}"; shift 2 ;;
    --archive-member) archive_member="${2:-}"; shift 2 ;;
    --strip-components) strip_components="${2:-}"; shift 2 ;;
    --dest) dest="${2:-}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
done

if [[ -z "$url" || -z "$sha256" || -z "$bin_name" ]]; then
  usage >&2
  exit 2
fi

if [[ ! "$sha256" =~ ^[0-9a-fA-F]{64}$ ]]; then
  echo "error: --sha256 must be 64 hex characters" >&2
  exit 2
fi

workdir="$(mktemp -d)"
cleanup() { rm -rf "$workdir"; }
trap cleanup EXIT

archive="${workdir}/archive"
curl --fail --silent --show-error --location --output "$archive" "$url"
got="$(sha256sum "$archive" | awk '{print $1}')"
expected="$(echo "$sha256" | tr 'A-F' 'a-f')"
if [[ "$got" != "$expected" ]]; then
  echo "error: checksum mismatch for ${url}" >&2
  echo "expected ${expected}" >&2
  echo "got      ${got}" >&2
  exit 1
fi

extract_dir="${workdir}/out"
mkdir -p "$extract_dir"
if [[ "$url" == *.zip ]]; then
  unzip -q "$archive" -d "$extract_dir"
else
  tar -xzf "$archive" -C "$extract_dir" --strip-components="$strip_components"
fi

if [[ -n "$archive_member" ]]; then
  src="${extract_dir}/${archive_member}"
else
  src="${extract_dir}/${bin_name}"
fi

if [[ ! -f "$src" ]]; then
  echo "error: binary not found at ${src}" >&2
  find "$extract_dir" -type f >&2 || true
  exit 1
fi

mkdir -p "$dest"
install -m 0755 "$src" "${dest}/${bin_name}"
echo "installed ${dest}/${bin_name}"
