#!/usr/bin/env bash
set -euo pipefail

# Parity check between `sisho tui` and `sisho-desk`.
# Compares 10 screens across all tabs on an identical vault in tmux virtual terminals.
#
# NOTE: This check is valid only while `sisho tui` exists (it will be removed in Phase 4).
# Because Phase 2 renamed the screen title to "Sisho Desk" and the welcome status to
# "Welcome to Sisho Desk!", this script normalizes those two strings on the `sisho tui`
# side before comparing.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

if [[ -z "${SISHO_BIN:-}" ]]; then
  echo "Error: SISHO_BIN environment variable must be set to the path of the sisho binary." >&2
  echo "Example: SISHO_BIN=/path/to/sisho $0" >&2
  exit 1
fi

if [[ ! -x "$SISHO_BIN" ]]; then
  echo "Error: sisho binary not found or not executable at '$SISHO_BIN'" >&2
  exit 1
fi

DESK_BIN="${DESK_BIN:-$REPO_DIR/target/debug/sisho-desk}"

if [[ ! -x "$DESK_BIN" ]]; then
  echo "Error: sisho-desk binary not found at $DESK_BIN. Run 'cargo build' first." >&2
  exit 1
fi

TMP_DIR="$(mktemp -d /tmp/sisho-parity-XXXXXX)"
VAULT_DIR="$TMP_DIR/vault"
mkdir -p "$VAULT_DIR"

# Populate a deterministic test vault
cat << 'EOF' > "$VAULT_DIR/default.config.tmt"
@config(
  workspace: {
    ignore: ["target/**", ".git/**"]
  }
)
EOF

cat << 'EOF' > "$VAULT_DIR/doc1.tmt"
@meta{
  author: "alice",
  kind: "guide"
}

# Document One

Content of document one.
EOF

cat << 'EOF' > "$VAULT_DIR/doc2.tmt"
@meta{
  author: "bob",
  kind: "spec"
}

# Document Two

Content of document two.
EOF

cat << 'EOF' > "$VAULT_DIR/legacy.md"
# Legacy Markdown Note

This note can be migrated to tomet.
EOF

SESSION_OLD="parity_old_$$"
SESSION_NEW="parity_new_$$"

cleanup() {
  tmux kill-session -t "$SESSION_OLD" 2>/dev/null || true
  tmux kill-session -t "$SESSION_NEW" 2>/dev/null || true
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

echo "Starting tmux sessions (80x24)..."
tmux new-session -d -s "$SESSION_OLD" -x 80 -y 24
tmux new-session -d -s "$SESSION_NEW" -x 80 -y 24

# Launch both programs
tmux send-keys -t "$SESSION_OLD" "$SISHO_BIN tui $VAULT_DIR" C-m
tmux send-keys -t "$SESSION_NEW" "$DESK_BIN $VAULT_DIR" C-m

sleep 1.0

step=1
compare_screen() {
  local desc="$1"
  local old_cap="$TMP_DIR/screen_${step}_old.txt"
  local new_cap="$TMP_DIR/screen_${step}_new.txt"

  tmux capture-pane -t "$SESSION_OLD" -p > "$old_cap"
  tmux capture-pane -t "$SESSION_NEW" -p > "$new_cap"

  # Normalize renamed strings on the `sisho tui` side:
  # "Sisho Desk" is 9 characters shorter than "Tomet Workbench TUI",
  # so Ratatui drew 9 more border dashes ('─') to fill the 80-column header.
  sed -i \
    -e '/Tomet Workbench TUI.*┐/s/─\{1,\}┐/─────────&/' \
    -e 's/Tomet Workbench TUI/Sisho Desk/g' \
    -e 's/Welcome to Tomet Workbench!/Welcome to Sisho Desk!/g' \
    "$old_cap"

  if diff -u "$old_cap" "$new_cap" > "$TMP_DIR/diff_${step}.patch"; then
    echo "  [Screen $step/10 OK] $desc"
  else
    echo "  [Screen $step/10 FAILED] $desc" >&2
    cat "$TMP_DIR/diff_${step}.patch" >&2
    exit 1
  fi
  step=$((step + 1))
}

echo "Running 10 screen parity checks..."

# Screen 1: Start screen (Explorer)
compare_screen "Start screen (Explorer)"

# Screen 2: Press '/' in Explorer (filter input)
tmux send-keys -t "$SESSION_OLD" "/"
tmux send-keys -t "$SESSION_NEW" "/"
sleep 0.2
compare_screen "Explorer filtering with '/'"

# Reset filter and switch to Migration
tmux send-keys -t "$SESSION_OLD" Escape
tmux send-keys -t "$SESSION_NEW" Escape
sleep 0.1
tmux send-keys -t "$SESSION_OLD" "2"
tmux send-keys -t "$SESSION_NEW" "2"
sleep 0.3

# Screen 3: Migration tab
compare_screen "Migration tab view"

# Screen 4: Select with Space, run confirmation with Enter
tmux send-keys -t "$SESSION_OLD" Space
tmux send-keys -t "$SESSION_NEW" Space
sleep 0.1
tmux send-keys -t "$SESSION_OLD" C-m
tmux send-keys -t "$SESSION_NEW" C-m
sleep 0.2
compare_screen "Migration confirmation modal"

# Cancel modal with 'n', switch to BatchMeta
tmux send-keys -t "$SESSION_OLD" "n"
tmux send-keys -t "$SESSION_NEW" "n"
sleep 0.1
tmux send-keys -t "$SESSION_OLD" "3"
tmux send-keys -t "$SESSION_NEW" "3"
sleep 0.3

# Screen 5: BatchMeta tab
compare_screen "BatchMeta tab view"

# Screen 6: BatchMeta 'k' (editing Key)
tmux send-keys -t "$SESSION_OLD" "k"
tmux send-keys -t "$SESSION_NEW" "k"
sleep 0.2
compare_screen "BatchMeta key edit focus"

# Screen 7: BatchMeta 'v' (editing Value)
tmux send-keys -t "$SESSION_OLD" "v"
tmux send-keys -t "$SESSION_NEW" "v"
sleep 0.2
compare_screen "BatchMeta value edit focus"

# Switch to StructuralGrep
tmux send-keys -t "$SESSION_OLD" Escape
tmux send-keys -t "$SESSION_NEW" Escape
sleep 0.1
tmux send-keys -t "$SESSION_OLD" "4"
tmux send-keys -t "$SESSION_NEW" "4"
sleep 0.3

# Screen 8: StructuralGrep tab
compare_screen "StructuralGrep tab view"

# Screen 9: StructuralGrep 't' (editing element condition)
tmux send-keys -t "$SESSION_OLD" "t"
tmux send-keys -t "$SESSION_NEW" "t"
sleep 0.2
compare_screen "StructuralGrep condition input ('t')"

# Screen 10: Quit with 'q' and check clean exit
tmux send-keys -t "$SESSION_OLD" Escape
tmux send-keys -t "$SESSION_NEW" Escape
sleep 0.1
tmux send-keys -t "$SESSION_OLD" "q"
tmux send-keys -t "$SESSION_NEW" "q"
sleep 0.5

# Verify both processes exited cleanly with code 0
# Send echo $? to check exit status in the shell
tmux send-keys -t "$SESSION_OLD" "echo EXIT_CODE:\$?" C-m
tmux send-keys -t "$SESSION_NEW" "echo EXIT_CODE:\$?" C-m
sleep 0.2
compare_screen "Clean exit after 'q'"

echo "All 10 screens and exit codes match identically! Parity check passed."
