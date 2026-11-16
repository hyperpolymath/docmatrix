# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
# shellcheck shell=bash
#
# provision-modes.sh — the launcher-standard 0.6.0 provisioning mode family,
# as a SOURCEABLE block shared by every launcher in the estate.
#
#   New launchers (library/tool/theory/docs): templates/launcher.sh.tmpl sources
#   this and calls hp_launcher_main.
#   Existing app launchers: add ONE line before their mode switch —
#
#       . "$REPO_DIR/build/just/provision-modes.sh" && hp_provision_or_return "$@"
#
#   A provisioning mode runs and the launcher exits with its status (a failed
#   --setup exits non-zero); any other mode returns, so the app's own switch
#   handles it. hp_provision_dispatch itself returns 99 for "not mine".
#
# Canon: hyperpolymath/standards launcher/launcher-standard_praxis.deed
#        (archetypes, provisioning-modes) and
#        3-practice/provisioning/PROVISIONING-STANDARD.adoc

HP_PROVISION_MODES_VERSION="0.6.0"

# Print the flat deed value for key $1, falling back to default $2.
hp__deed() { # $1 key, $2 default — flat (key "value") read from provisioning_praxis.deed
  local f="$REPO_DIR/.machine_readable/descriptiles/provisioning_praxis.deed" v=""
  [ -f "$f" ] && v=$(grep -oE "[(:]$1[[:space:]]+\"[^\"]*\"" "$f" | head -1 | sed -E 's/^[^"]*"//; s/"$//')
  printf '%s' "${v:-$2}"
}

# Print the declared repository archetype, defaulting to library.
hp_archetype() { hp__deed archetype "library"; }
# Without a deed, the origin remote names the repo (a worktree or renamed clone
# has another directory name); the directory is the last resort.
hp_app_name()  {
  local u; u=$(git -C "$REPO_DIR" config --get remote.origin.url 2>/dev/null || true)
  u=${u%.git}; u=${u##*/}
  hp__deed name "${u:-$(basename "$REPO_DIR")}"
}

# Print linux, macos, windows or unknown based on the host kernel name.
hp_platform() {
  case "$(uname -s)" in
    Linux*)                          echo linux ;;
    Darwin*)                         echo macos ;;
    CYGWIN*|MINGW*|MSYS*|Windows_NT) echo windows ;;
    *)                               echo unknown ;;
  esac
}

# Make sure `just` is runnable: PATH, then mise, then say exactly what to do.
# May install just globally via mise, extend PATH or define a just wrapper.
# Return 0 when available, otherwise 1 after printing installation guidance.
hp_ensure_just() {
  command -v just >/dev/null 2>&1 && return 0
  if command -v mise >/dev/null 2>&1; then
    echo "just is not installed — installing it with mise (mise use -g just@latest)..." >&2
    mise use -g just@latest >&2 && PATH="$(mise bin-paths 2>/dev/null | paste -sd: -):$PATH" && command -v just >/dev/null 2>&1 && return 0
    mise exec just@latest -- true >/dev/null 2>&1 && { just() { mise exec just@latest -- just "$@"; }; return 0; }
  fi
  cat >&2 <<'EOF'
Neither `just` nor `mise` is installed. Install mise (it then installs everything else):

    brew install mise                    # macOS
    sudo dnf copr enable jdxcode/mise && sudo dnf install mise   # Fedora
    winget install jdx.mise              # Windows
    # anything else (Debian/Ubuntu apt, …): https://mise.jdx.dev/installing-mise.html
    # or download the installer, read it, then run it — never pipe it into a shell:
    curl -fsSLo mise-install.sh https://mise.run && less mise-install.sh && sh mise-install.sh

then re-run this command. Manual route without mise: docs/SETUP.adoc
EOF
  return 1
}

# Ensure just is available, then run it with the supplied arguments in REPO_DIR.
hp_just() { hp_ensure_just || return 1; (cd "$REPO_DIR" && just "$@"); }
# The provisioning modes call the engine directly, not a `just` recipe: a repo may
# define its own root `doctor`/`setup`/`heal`, and the launcher must still run the
# canon (which then runs that repo's *-local recipes). Only needs bash.
hp_lib() { (cd "$REPO_DIR" && bash build/just/provision-lib.sh "$@"); }

# The one-line hook for existing launchers: exit with a provisioning mode's
# status, or return (0) so the caller's own mode switch runs.
hp_provision_or_return() {
  hp_provision_dispatch "$@"
  local rc=$?
  [ "$rc" -eq 99 ] && return 0
  exit "$rc"
}

# Run the provisioning mode named by $1 and return its status;
# return 99 when the mode is not handled here.
hp_provision_dispatch() {
  case "${1:-}" in
    --setup)    hp_lib setup ;;
    --doctor)   hp_lib doctor ;;
    --heal)     hp_lib heal ;;
    --ai-setup) hp_lib ai-setup ;;
    *)          return 99 ;;
  esac
}

# Print the launcher name, version, commit, platform and architecture.
hp_version_line() {
  local sha ver
  sha=$(git -C "$REPO_DIR" rev-parse --short HEAD 2>/dev/null || echo unknown)
  ver=$(hp__deed version "")
  [ -z "$ver" ] && ver=$(git -C "$REPO_DIR" describe --tags --abbrev=0 2>/dev/null || echo 0.0.0)
  printf '%s-launcher %s (%s) [%s-%s]\n' "$(hp_app_name)" "${ver#v}" "$sha" "$(hp_platform)" "$(uname -m)"
}

# Print launcher usage and the modes applicable to the repository archetype.
hp_help() {
  local name arch; name=$(hp_app_name); arch=$(hp_archetype)
  cat <<EOF
$name launcher (launcher-standard $HP_PROVISION_MODES_VERSION, archetype: $arch)

Usage: ./launcher.sh <mode>

Provisioning (every repository):
  --setup      Install everything this repository needs, then run --doctor
  --doctor     Check the environment; PASS/WARN/FAIL with fix hints (exit 1 on FAIL)
  --heal       Apply safe fixes automatically, then re-run --doctor
  --ai-setup   Print the one line to give any AI assistant to set this up for you

Meta:
  --help       This text
  --version    Machine-readable version line

EOF
  if [ "$arch" = app ]; then
    echo "Runtime: --start --stop --status --auto (default) --integ --disinteg"
  else
    echo "Runtime modes (--start --stop --status --auto --integ --disinteg) do not apply"
    echo "to a $arch repository; they are accepted and explain themselves."
  fi
  cat <<EOF

Everything here is also a just recipe: run 'just' to list them all.
Manual setup, step by step: docs/SETUP.adoc
Detected platform: $(hp_platform)
EOF
}

# Whole-launcher main for non-app archetypes; use --help when $1 is absent.
# Return the provisioning status, 2 for unknown modes, or 1 for an app runtime
# request. Accepted runtime modes for other archetypes explain N/A and return 0.
hp_launcher_main() {
  local mode=${1:---help} rc
  case "$mode" in
    -h|--help)    hp_help; return 0 ;;
    -V|--version) hp_version_line; return 0 ;;
  esac
  hp_provision_dispatch "$mode"; rc=$?
  [ $rc -ne 99 ] && return $rc
  case "$mode" in
    --start|--stop|--status|--auto|--browser|--web|--integ|--disinteg|--debug|--logs|--tail)
      local arch; arch=$(hp_archetype)
      if [ "$arch" = app ]; then
        echo "$(hp_app_name) declares archetype 'app' but this launcher has no runtime modes — regenerate it with launch-scaffolder." >&2
        return 1
      fi
      echo "$(hp_app_name) is a $arch: there is nothing to ${mode#--}. Try --setup, --doctor or --help."
      return 0 ;;
    *)
      echo "Unknown mode: $mode" >&2; hp_help >&2; return 2 ;;
  esac
}
