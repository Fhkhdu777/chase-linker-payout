#!/usr/bin/env bash
set -euo pipefail

# Installs the system packages needed to compile the Rust project.
# Supports apt (Debian/Ubuntu), dnf (Fedora/RHEL/CentOS), and pacman (Arch).

if [[ ${EUID:-$(id -u)} -ne 0 ]]; then
  SUDO="sudo"
else
  SUDO=""
fi

command_exists() {
  command -v "$1" >/dev/null 2>&1
}

log() {
  printf '[setup] %s\n' "$1"
}

if command_exists apt-get; then
  log 'Detected apt-based distribution.'
  export DEBIAN_FRONTEND=noninteractive
  log 'Updating package index...'
  $SUDO apt-get update
  log 'Installing build-essential pkg-config libssl-dev...'
  $SUDO apt-get install --yes build-essential pkg-config libssl-dev
elif command_exists dnf; then
  log 'Detected dnf-based distribution.'
  log 'Installing development tools group and dependencies...'
  $SUDO dnf groupinstall --yes 'Development Tools'
  $SUDO dnf install --yes pkg-config openssl-devel
elif command_exists pacman; then
  log 'Detected pacman-based distribution.'
  log 'Installing base-devel pkgconf openssl...'
  $SUDO pacman -Sy --noconfirm base-devel pkgconf openssl
else
  printf 'Unsupported package manager. Please install a C toolchain, pkg-config, and OpenSSL headers manually.\n' >&2
  exit 1
fi

if command_exists cc; then
  log "C toolchain available: $(cc --version | head -n1)"
else
  printf 'cc still not found. Verify your installation manually.\n' >&2
  exit 1
fi

log 'System dependencies installed successfully.'
