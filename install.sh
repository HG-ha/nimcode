#!/usr/bin/env bash
set -euo pipefail

REPO="HG-ha/nimcode"
BINARY="nimcode"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

get_latest_version() {
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" |
    grep '"tag_name"' | head -1 | sed -E 's/.*"v([^"]+)".*/\1/'
}

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)  os="unknown-linux-gnu" ;;
    Darwin) os="apple-darwin" ;;
    *)      echo "Unsupported OS: $os" >&2; exit 1 ;;
  esac

  case "$arch" in
    x86_64|amd64)  arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *)             echo "Unsupported arch: $arch" >&2; exit 1 ;;
  esac

  echo "${arch}-${os}"
}

main() {
  local version target url tmpdir

  echo "NimCode installer"
  echo ""

  version="${1:-$(get_latest_version)}"
  if [ -z "$version" ]; then
    echo "Error: could not detect latest version. Pass a version number as argument." >&2
    exit 1
  fi

  target="$(detect_target)"
  url="https://github.com/${REPO}/releases/download/v${version}/nimcode-${target}.tar.gz"

  echo "  Version : v${version}"
  echo "  Target  : ${target}"
  echo "  Install : ${INSTALL_DIR}/${BINARY}"
  echo ""

  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT

  echo "Downloading ${url}..."
  curl -fsSL "$url" -o "${tmpdir}/nimcode.tar.gz"

  echo "Extracting..."
  tar xzf "${tmpdir}/nimcode.tar.gz" -C "$tmpdir"

  echo "Installing to ${INSTALL_DIR}/${BINARY}..."
  if [ -w "$INSTALL_DIR" ]; then
    mv "${tmpdir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
  else
    sudo mv "${tmpdir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
  fi
  chmod +x "${INSTALL_DIR}/${BINARY}"

  echo ""
  echo "✓ nimcode v${version} installed to ${INSTALL_DIR}/${BINARY}"
  echo ""
  echo "Run 'nimcode' to get started. It will prompt for your NVIDIA NIM API Key on first launch."
}

main "$@"
