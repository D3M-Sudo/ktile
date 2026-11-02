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
    stop_child() {
        local pid="$1"
        [[ -n "$pid" ]] || return 0
        kill -TERM "$pid" 2>/dev/null || true
        for _ in $(seq 1 20); do
            kill -0 "$pid" 2>/dev/null || break
            sleep 0.1
        done
        kill -KILL "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
    }
    stop_child "$WATCH_PID"
    stop_child "$WINDOW_PID"
    stop_child "$WM_PID"
    stop_child "$XVFB_PID"
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

command -v Xvfb >/dev/null
command -v openbox >/dev/null
command -v xmessage >/dev/null
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

# Verify custom TOML settings are actually consumed by the layout preview.
printf 'gap = 0\n' >"$TMP_DIR/custom.toml"
DISPLAY="$DISPLAY" target/release/k-tile plan --width 800 --height 600 --windows 3 --layout grid >"$TMP_DIR/default-plan.log"
DISPLAY="$DISPLAY" target/release/k-tile --config "$TMP_DIR/custom.toml" plan --width 800 --height 600 --windows 3 --layout grid >"$TMP_DIR/custom-plan.log"
if cmp -s "$TMP_DIR/default-plan.log" "$TMP_DIR/custom-plan.log"; then
    echo "Custom TOML gap setting did not affect the layout preview." >&2
    exit 1
fi

xmessage -title "K-tile integration test" "K-tile X11 integration test window" >"$TMP_DIR/xmessage.log" 2>&1 &
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

# Live planning reads actual client metadata but must never change geometry.
DISPLAY="$DISPLAY" xdotool getwindowgeometry --shell "$WINDOW_ID" \
    | grep -E '^(X|Y|WIDTH|HEIGHT)=' >"$TMP_DIR/geometry-before.log"
DISPLAY="$DISPLAY" target/release/k-tile plan-live >"$TMP_DIR/live-plan.log" 2>&1
grep -q "read-only live preview" "$TMP_DIR/live-plan.log"
grep -q "K-tile integration test" "$TMP_DIR/live-plan.log"
grep -q "No window geometry was changed." "$TMP_DIR/live-plan.log"
DISPLAY="$DISPLAY" xdotool getwindowgeometry --shell "$WINDOW_ID" \
    | grep -E '^(X|Y|WIDTH|HEIGHT)=' >"$TMP_DIR/geometry-after.log"
if ! cmp -s "$TMP_DIR/geometry-before.log" "$TMP_DIR/geometry-after.log"; then
    echo "Read-only plan-live changed the test window geometry." >&2
    diff -u "$TMP_DIR/geometry-before.log" "$TMP_DIR/geometry-after.log" >&2 || true
    exit 1
fi

# Verify the diagnostic watcher observes an externally initiated configure event.
DISPLAY="$DISPLAY" target/release/k-tile watch-x11 >"$TMP_DIR/watch.log" 2>&1 &
WATCH_PID=$!
sleep 0.4
DISPLAY="$DISPLAY" xdotool windowmove "$WINDOW_ID" 160 120
sleep 0.6
kill -TERM "$WATCH_PID"
if ! timeout 5s tail --pid="$WATCH_PID" -f /dev/null; then
    kill -KILL "$WATCH_PID" 2>/dev/null || true
    wait "$WATCH_PID" 2>/dev/null || true
    cat "$TMP_DIR/watch.log" >&2
    echo "watch-x11 did not exit within 5 seconds after SIGTERM." >&2
    exit 1
fi
if ! wait "$WATCH_PID"; then
    cat "$TMP_DIR/watch.log" >&2
    echo "watch-x11 did not exit cleanly after SIGTERM." >&2
    exit 1
fi
WATCH_PID=""

if ! grep -q "X11 watcher received cooperative shutdown request" "$TMP_DIR/watch.log"; then
    cat "$TMP_DIR/watch.log" >&2
    echo "watch-x11 did not report cooperative shutdown." >&2
    exit 1
fi
if ! grep -q "X11 window geometry changed" "$TMP_DIR/watch.log"; then
    cat "$TMP_DIR/watch.log" >&2
    echo "X11 watcher did not report the test window geometry change." >&2
    exit 1
fi
if ! grep -q "processed coalesced X11 event batch" "$TMP_DIR/watch.log"; then
    cat "$TMP_DIR/watch.log" >&2
    echo "X11 watcher did not report coalesced event-batch processing." >&2
    exit 1
fi
if grep -oE 'events=[0-9]+' "$TMP_DIR/watch.log" | cut -d= -f2 | awk '$1 > 128 { found=1 } END { exit !found }'; then
    cat "$TMP_DIR/watch.log" >&2
    echo "X11 watcher exceeded the 128-event batch bound." >&2
    exit 1
fi

# The event-driven planner should refresh exactly in response to a relevant batch.
DISPLAY="$DISPLAY" target/release/k-tile watch-plan >"$TMP_DIR/watch-plan.log" 2>&1 &
WATCH_PID=$!
sleep 0.4
DISPLAY="$DISPLAY" xdotool windowmove "$WINDOW_ID" 190 140
for _ in $(seq 1 30); do
    if grep -q 'reconcile_needed=true' "$TMP_DIR/watch-plan.log"; then break; fi
    sleep 0.1
done
kill -TERM "$WATCH_PID"
if ! timeout 5s tail --pid="$WATCH_PID" -f /dev/null; then
    kill -KILL "$WATCH_PID" 2>/dev/null || true
    wait "$WATCH_PID" 2>/dev/null || true
    cat "$TMP_DIR/watch-plan.log" >&2
    echo "watch-plan did not exit within 5 seconds after SIGTERM." >&2
    exit 1
fi
if ! wait "$WATCH_PID"; then
    cat "$TMP_DIR/watch-plan.log" >&2
    echo "watch-plan did not exit cleanly after SIGTERM." >&2
    exit 1
fi
WATCH_PID=""
if ! grep -q "X11 watcher received cooperative shutdown request" "$TMP_DIR/watch-plan.log"; then
    cat "$TMP_DIR/watch-plan.log" >&2
    echo "watch-plan did not report cooperative shutdown." >&2
    exit 1
fi
if ! grep -q 'processed coalesced X11 event batch' "$TMP_DIR/watch-plan.log" \
    || [[ "$(grep -c '^read-only live preview:' "$TMP_DIR/watch-plan.log")" -lt 2 ]]; then
    cat "$TMP_DIR/watch-plan.log" >&2
    echo "watch-plan did not trigger a preview refresh after a window event." >&2
    exit 1
fi

echo "X11 integration checks passed (Xvfb + Openbox; geometry mutation remains disabled)."
