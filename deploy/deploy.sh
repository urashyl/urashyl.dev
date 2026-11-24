#!/usr/bin/env bash
# Build the release binary and (re)start the systemd user service.
# Safe to run repeatedly: use it for the first install and for every update.
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
expected="$(cd "$HOME" && pwd -P)/urashyl.dev"
unit="urashyl-dev.service"

if [[ "$repo" != "$expected" ]]; then
  echo "The service runs from ~/urashyl.dev, but this checkout is at $repo." >&2
  echo "Move it with: mv \"$repo\" ~/urashyl.dev" >&2
  exit 1
fi

cd "$repo"
cargo build --release

# systemctl --user finds the per-user systemd instance through XDG_RUNTIME_DIR
# (its socket is $XDG_RUNTIME_DIR/systemd/private; the D-Bus "bus" socket only
# exists if dbus-user-session is installed). `su user` without `-` keeps root's
# XDG_RUNTIME_DIR=/run/user/0, which fails with "Operation not permitted".
runtime_dir="/run/user/$(id -u)"
export XDG_RUNTIME_DIR="$runtime_dir"
if [[ -S "$runtime_dir/bus" ]]; then
  export DBUS_SESSION_BUS_ADDRESS="unix:path=$runtime_dir/bus"
else
  unset DBUS_SESSION_BUS_ADDRESS
fi

if [[ ! -S "$runtime_dir/systemd/private" ]]; then
  echo "No systemd user manager is running for $USER ($runtime_dir/systemd/private is missing)." >&2
  echo "As root, check it with: systemctl status user@$(id -u).service" >&2
  echo "Start one that also survives logout and reboot with:" >&2
  echo "  sudo loginctl enable-linger $USER" >&2
  echo "then run this script again." >&2
  exit 1
fi

install -Dm644 "deploy/$unit" "$HOME/.config/systemd/user/$unit"
systemctl --user daemon-reload
systemctl --user enable "$unit" >/dev/null
systemctl --user restart "$unit"
sleep 1
systemctl --user --no-pager --lines=5 status "$unit"

if command -v loginctl >/dev/null && ! loginctl show-user "$USER" -p Linger 2>/dev/null | grep -q '=yes'; then
  echo
  echo "Lingering is off, so the service stops when you log out and won't start at boot."
  echo "Turn it on with: sudo loginctl enable-linger $USER"
fi
