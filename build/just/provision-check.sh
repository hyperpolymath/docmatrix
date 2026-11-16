#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
#
# provision-check.sh — offline conformance check for the provisioning set
# (PROVISIONING-STANDARD.adoc §8, items 1–5). No network, no installs.
#
#   provision-check.sh [--dev] [REPO_DIR]
#
# Exit 0 = conforms, 1 = does not. Every failure is printed; none is skipped.
# --dev downgrades repository-specific slot residue (__SPEC_X__) to a warning,
# for a checkout mid-specialisation. Mechanical residue (__X__) always fails.
set -uo pipefail

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
LIB="$HERE/provision-lib.sh"
DEV=0
[ "${1:-}" = "--dev" ] && { DEV=1; shift; }
REPO="${1:-.}"
cd "$REPO" || { echo "provision-check: no such directory: $REPO" >&2; exit 1; }

MIN_JUST="1.42.0"
VERBS="setup doctor heal dev-shell toolchain-refresh ai-setup ai-warmup eval config-show opsm build test bench lint fmt fmt-check run deps"  # hypatia:ignore eval_in_shell -- "eval" is a verb name, not the shell builtin
# Files the provisioning set owns; residue anywhere else is not ours to judge.
# The engine scripts under build/just/ are excluded: they name slot syntax in their
# own comments and code, and realign owns them. README.adoc is scanned only inside
# its [[ai-install]] section.
# Their locations come from provision-lib.sh (guix-dir, set-files): the engine and
# this check must never disagree about where the Guix files or warm-ups live.
[ -f "$LIB" ] || { echo "provision-check: engine missing: $LIB" >&2; exit 1; }
# Run the provisioning engine with the checked repository as its root.
lib() { PROVISION_ROOT="$PWD" bash "$LIB" "$@"; }
GDIR=$(lib guix-dir)
SET_FILES=$(lib set-files)
# Print the path to Guix filename $1 using the resolved Guix directory.
gp() { [ "$GDIR" = . ] && echo "$1" || echo "$GDIR/$1"; }

fails=0 warns=0
# Print a successful conformance check.
ok()   { printf '  ok    %s\n' "$*"; }
# Print a failed conformance check and increment the failure count.
bad()  { printf '  FAIL  %s\n' "$*"; fails=$((fails + 1)); }
# Print a conformance warning and increment the warning count.
warn() { printf '  warn  %s\n' "$*"; warns=$((warns + 1)); }

# Return success when version $1 is at least $2, using version sort order.
version_ge() { [ "$(printf '%s\n%s\n' "$2" "$1" | sort -V | head -1)" = "$2" ]; }

echo "[1] launcher"
if [ ! -f launcher.sh ]; then bad "launcher.sh missing"
elif [ ! -x launcher.sh ]; then bad "launcher.sh is not executable (chmod +x; committed mode must be 100755)"
else
  for m in --help --version; do
    if ./launcher.sh "$m" >/dev/null 2>&1; then ok "launcher.sh $m exits 0"
    else bad "launcher.sh $m exits non-zero"; fi
  done
fi

echo "[2] just"
f0=$fails
if ! command -v just >/dev/null 2>&1; then bad "just is not installed; cannot check the recipe contract"
else
  jv=$(just --version 2>/dev/null | awk '{print $2}')
  if version_ge "$jv" "$MIN_JUST"; then ok "just $jv >= $MIN_JUST"
  else bad "just $jv < $MIN_JUST (module-recipe dependencies do not resolve)"; fi
  if ! summary=$(just --summary 2>&1); then
    bad "the Justfile does not parse: $(printf '%s' "$summary" | head -1)"
  else
    have=" $summary "
    for v in $VERBS; do
      case "$have" in *" $v "*) ;; *) bad "root recipe missing: $v" ;; esac
      case "$have" in *" provision::$v "*) ;; *) bad "module recipe missing: provision::$v (engine drift)" ;; esac
    done
    [ "$fails" -eq "$f0" ] && ok "all ${VERBS// /, } present at root and in provision::"
  fi
fi

echo "[3] mise"
if [ ! -f mise.toml ]; then bad "mise.toml missing"
else
  if hit=$(lib mise-banned); then ok "mise.toml names no banned tool"; else bad "mise.toml names banned tools: $hit"; fi
  [ -f .mise.toml ] && bad "both mise.toml and .mise.toml exist"
  if lg=$(lib mise-lock-gaps); then ok "mise.lock pins every mise.toml tool"; else bad "$lg (latest is not concrete; run: mise lock)"; fi
fi

echo "[4] guix"
for g in "$(gp guix.scm)" "$(gp manifest.scm)" "$(gp channels.scm)"; do
  if r=$(lib guix-stub "$g"); then ok "$g is not a stub"; else bad "$g: $r"; fi
done

echo "[5] template residue"
f0=$fails
# Read text from stdin and report unfilled slots under label $1;
# repository-specific slots are warnings in --dev mode and failures otherwise.
# Mechanical slots always fail. Callers use the tallies, not the return status.
residue() { # $1 label; stdin = the text to judge
  local text mech spec
  text=$(cat)
  # Mechanical slots (__X__ not starting SPEC_) are the generator's job: always fatal.
  mech=$(printf '%s\n' "$text" | grep -noE '__[A-Z][A-Z_]*__' | grep -v ':__SPEC_' | head -3 | tr '\n' ' ')
  spec=$(printf '%s\n' "$text" | grep -noE '__SPEC_[A-Z_]*__' | head -3 | tr '\n' ' ')
  [ -n "$mech" ] && bad "$1: unfilled mechanical slots: $mech"
  if [ -n "$spec" ]; then
    if [ "$DEV" = 1 ]; then warn "$1: unfilled repository slots: $spec"
    else bad "$1: unfilled repository slots: $spec"; fi
  fi
}
for f in $SET_FILES; do
  # shellcheck disable=SC2094 # $f is only the label; nothing writes it
  [ -f "$f" ] && residue "$f" < "$f"
done
for r in README.adoc README.md; do
  [ -f "$r" ] || continue
  # From the [[ai-install]] anchor to the next level-2 heading after its own.
  residue "$r [[ai-install]]" < <(awk '/^\[\[ai-install\]\]/{on=1; n=0} on && /^== /{n++; if (n>1) exit} on' "$r")
done
[ "$fails" -eq "$f0" ] && ok "no residue that fails this mode"

echo
echo "provision-check: $fails FAIL, $warns WARN"
[ "$fails" -eq 0 ]
