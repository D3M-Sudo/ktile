#!/usr/bin/env bash
set -Eeuo pipefail

# Run only against a disposable X server; never target the user's live DISPLAY.
: "${DISPLAY:=:99}"
export DISPLAY
export RUST_LOG="${RUST_LOG:-debug}"

XVFB_PID=""
WM_PID=""
WINDOW_PID=""
WATCH_PID=""
TMP_DIR="$(mktemp -d)"

cleanup() {
    set +e
    [[ -n "$WATCH_PID" ]] && kill "$WATCH_PID" 2>/dev/null
    [[ -n "$WINDOW_PID" ]] && kill "$WINDOW_PID" 2>/dev/null
    [[ -n "$WM_PID" ]] && kill "$WM_PID" 2>/dev/null
    [[ -n "$XVFB_PID" ]] && kill "$XVFB_PID" 2>/dev/null
    wait 2>/dev/null
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

command -v Xvfb >/dev/null
command -v openbox >/dev/null
command -v xterm >/dev/null
command -v xdotool >/dev/null
[[ -x target/release/k-tile ]] || {
    echo "Build target/release/k-tile before running this test." >&2
    exit 1
}

# The script owns this display. Refuse to start if it is already occupied.
if [[ -S "/tmp/.X11-unix/X${DISPLAY#:}" ]]; then
    echo "Refusing to use occupied display $DISPLAY; choose a free display." >&2
    exit 1
fi

Xvfb "$DISPLAY" -screen 0 1280x800x24 -nolisten tcp +extension RANDR >"$TMP_DIR/xvfb.log" 2>&1 &
XVFB_PID=$!
for _ in $(seq 1 50); do
    if DISPLAY="$DISPLAY" xdotool getdisplaygeometry >/dev/null 2>&1; then break; fi
    sleep 0.1
done
DISPLAY="$DISPLAY" xdotool getdisplaygeometry >/dev/null

openbox >"$TMP_DIR/openbox.log" 2>&1 &
WM_PID=$!
for _ in $(seq 1 50); do
    if DISPLAY="$DISPLAY" target/release/k-tile check-x11 >"$TMP_DIR/check.log" 2>&1; then break; fi
    sleep 0.1
done
DISPLAY="$DISPLAY" target/release/k-tile check-x11 >"$TMP_DIR/check.log" 2>&1 || {
    cat "$TMP_DIR/check.log" >&2
    exit 1
}
grep -q "X11 connection OK" "$TMP_DIR/check.log"

xterm -title "K-tile integration test" >"$TMP_DIR/xterm.log" 2>&1 &
WINDOW_PID=$!
WINDOW_ID=""
for _ in $(seq 1 50); do
    WINDOW_ID="$(DISPLAY="$DISPLAY" xdotool search --onlyvisible --name "K-tile integration test" 2>/dev/null | head -n1 || true)"
    [[ -n "$WINDOW_ID" ]] && break
    sleep 0.1
done
[[ -n "$WINDOW_ID" ]] || {
    echo "Test window did not become visible." >&2
    exit 1
}

DISPLAY="$DISPLAY" target/release/k-tile inspect-windows >"$TMP_DIR/windows.log" 2>&1
grep -q "K-tile integration test" "$TMP_DIR/windows.log"

# Verify the diagnostic watcher observes an externally initiated configure event.
DISPLAY="$DISPLAY" timeout 5s target/release/k-tile watch-x11 >"$TMP_DIR/watch.log" 2>&1 &
WATCH_PID=$!
sleep 0.4
DISPLAY="$DISPLAY" xdotool windowmove "$WINDOW_ID" 160 120
sleep 0.6
kill "$WATCH_PID" 2>/dev/null || true
wait "$WATCH_PID" 2>/dev/null || true
WATCH_PID=""

if ! grep -q "X11 window geometry changed" "$TMP_DIR/watch.log"; then
    cat "$TMP_DIR/watch.log" >&2
    echo "X11 watcher did not report the test window geometry change." >&2
    exit 1
fi

echo "X11 integration checks passed (Xvfb + Openbox; no live desktop windows modified)."
