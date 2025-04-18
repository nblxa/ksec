#!/bin/sh

set -eu

_get_arch() {
  if [ "$1" = "x86_64" ]; then
    echo "$2"
  elif [ "$1" = "arm64" ]; then
    echo "$3"
  else
    echo "Unsupported architecture: '$1'. Exiting..." >&2
    exit 1
  fi
}
_ks_install() {
  echo "Installing ksec..."
  _ks_acc="nblxa"
  _ks_repo="ksec"
  _ks_bin="ksec"
  _ks_ver="${KSEC_VERSION:-v0.1.0}"
  _ks_ext="${KSEC_EXT:-}"
  _ks_arch="${KSEC_ARCH:-}"
  _ks_os="${KSEC_OS:-}"
  if [ -z "${_ks_os:-}" ]; then
    _ks_uname_s="$(uname -s)"
    if [ "$_ks_uname_s" = "Linux" ]; then
      _ks_os="unknown-linux-gnu"
      _ks_ext="${_ks_ext:-tar.gz}"
    elif [ "$_ks_uname_s" = "Darwin" ]; then
      _ks_os="apple-darwin"
      _ks_ext="${_ks_ext:-tar.gz}"
    elif [ "$_ks_uname_s" = "Windows" ]; then
      _ks_os="pc-windows-msvc"
      _ks_ext="${ks_ext:-zip}"
    else
      echo "Unsupported OS: '$_ks_uname_s'. Exiting..." >&2
      exit 1
    fi
  fi
  if [ -z "${_ks_arch:-}" ]; then
    _ks_uname_m="$(uname -m)"
    _ks_arch="$(_get_arch "$_ks_uname_m" "x86_64" "aarch64")"
  fi
  if [ -z "${_ks_arch:-}" ]; then
    echo "KSEC_ARCH is not set and couldn't determine the architecture. Exiting..." >&2
    exit 1
  fi
  if [ -z "${_ks_ext:-}" ]; then
    if [ "$_ks_os" = "pc-windows-msvc" ]; then
      _ks_ext="zip"
    else
      _ks_ext="tar.gz"
    fi
  fi
  if [ -z "${HOME:-}" ]; then
    echo "HOME is not set. Exiting..." >&2
    exit 1
  fi
  _ks_dir="$HOME/.$_ks_bin/bin"
  mkdir -p "$_ks_dir"
  _ks_url="https://github.com/$_ks_acc/$_ks_repo/releases/download/$_ks_ver/$_ks_bin-$_ks_arch-$_ks_os.$_ks_ext"
  _ks_path="$_ks_dir/$_ks_bin.$_ks_ext"
  if command -v curl >/dev/null 2>&1; then
    curl -sSLf -o "$_ks_path" "$_ks_url"
  elif command -v wget >/dev/null 2>&1; then
    rc=0
    wget --quiet --https-only -O "$_ks_path" "$_ks_url" || rc=$?
    case $rc in
      0)
        ;;
      8)
        echo "Server issued an error. Exiting..." >&2
        exit 1
        ;;
      4)
        echo "Network failure. Exiting..." >&2
        exit 1
        ;;
      *)
        echo "wget error: $rc. Exiting..." >&2
        exit 1
        ;;
    esac
  fi
  cd "$_ks_dir"
  tar -xzf "$_ks_bin.$_ks_ext"
  chmod +x "$_ks_bin"
  rm -rf "$_ks_bin.$_ks_ext"
  # if PATH doesn't contain $_ks_dir, add it
  if ! echo ":$PATH:" | grep -q ":$_ks_dir:"; then
    PATH="$_ks_dir:$PATH"
    export PATH
  fi
  if [ -f "$HOME/.bashrc" ]; then
    if ! grep -q "export PATH=\"$_ks_dir:\$PATH\"" "$HOME/.bashrc"; then
      echo "export PATH=\"$_ks_dir:\$PATH\"" >> "$HOME/.bashrc"
    fi
  fi
  if [ -f "$HOME/.zshrc" ]; then
    if ! grep -q "export PATH=\"$_ks_dir:\$PATH\"" "$HOME/.zshrc"; then
      echo "export PATH=\"$_ks_dir:\$PATH\"" >> "$HOME/.zshrc"
    fi
  fi
  echo "ksec installed successfully."
}
_ks_install
