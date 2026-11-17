#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
#
# check-lock-sync.sh — verify .github/workflows/actions.lock is in sync with the
# workflow YAML, in BOTH directions, including job-level reusable-workflow refs.
#
# This is the estate-wide template from metadatastician/standards for issue #968.
# Adapted from metadatastician/burble#224.
#
# Why this exists rather than `gh actions-lock --verify-local` alone:
#   `gh actions-lock` v0.1.6 cannot see a job-level `uses:` (a reusable-workflow
#   call); v0.2.0 still cannot (re-measured 2026-10-14). Measured on this repo, 2026-09-22:
#     * mirror.yml's lock entry was stale by a whole SHA and died at startup with
#       jobs=0, yet --verify-local reported "All 30 workflows have complete
#       lockfile coverage" and fix mode did not repair it;
#     * release.yml's job-level slsa-github-generator ref was absent from the
#       lock, again unreported; and once added, --verify-local calls it `stale`
#       with "no uses: in this workflow references it" — about a ref on line 144.
#   GitHub's own startup enforcement DOES check those refs (that is what killed
#   mirror.yml), so the tool is wrong in both directions and cannot be the
#   authority. This script is.
#
# A workflow's `uses:` set includes the `uses:` of every LOCAL composite action
# it calls (`uses: ./.github/actions/x`), transitively: a local action has no
# dependency record of its own, so its actions are locked under the calling
# workflow, which is where `gh actions-lock` writes them (standards#1122).
#
# Exit 0 only when every workflow's `uses:` set equals its lockfile set exactly.
# Any mismatch exits 1. There is no warn-only mode: a desync means GitHub will
# refuse to start the run, so it must fail the job.
# Requires Bash 4+, jq and Mike Farah yq v4; see docs/actions-lock-validation.adoc.

set -euo pipefail

WF_DIR="${1:-.github/workflows}"
LOCK="$WF_DIR/actions.lock"
# The reviewed closure baseline (check-reusable-closure.sh --write-baseline).
# Optional: without it this gate only knows the refs named in the YAML, which is
# the behaviour it had before; with it, a through-ref recorded under the calling
# workflow is recognised as referenced in both directions.
CLOSURE="$WF_DIR/actions-lock.closure.tsv"
[ -f "$CLOSURE" ] || CLOSURE=""

if [ ! -f "$LOCK" ]; then
  echo "check-lock-sync: FATAL: no lockfile at $LOCK" >&2
  exit 1
fi

shopt -s nullglob
mapfile -t WORKFLOWS < <(printf '%s\n' "$WF_DIR"/*.yml "$WF_DIR"/*.yaml | sort -u)
if [ "${#WORKFLOWS[@]}" -eq 0 ]; then
  echo "check-lock-sync: FATAL: no workflow files under $WF_DIR" >&2
  exit 1
fi

# ── local composite actions (identical block in check-lock-sync.sh, ─────────
# ── prune-stale.sh and complete-job-refs.sh; they must agree) ─────────────────
# reach_files WORKFLOW prints WORKFLOW and every local composite action file
# (`uses: ./dir` -> dir/action.yml or dir/action.yaml) it reaches, transitively,
# each once. A local reusable workflow (`./….yml`) is a lock entry of its own,
# so it is not followed. Paths resolve from the repository root, as GitHub
# resolves them from the workspace.
LC_ROOT="$(cd "$WF_DIR/../.." 2>/dev/null && pwd)" || LC_ROOT=.
# Parse one YAML document and reject duplicate mapping keys before converting
# to JSON. yq exposes node keys without discarding duplicates; JSON alone does
# not. jq handles typed fields after YAML comments, aliases and quoting decode.
lock_yaml_json() {
  command -v yq >/dev/null && command -v jq >/dev/null || {
    echo 'actions-lock: requires Mike Farah yq v4 and jq' >&2; return 1;
  }
  yq --version | grep -Eq 'github.com/mikefarah/yq/.*version v4\.' || {
    echo 'actions-lock: requires Mike Farah yq v4' >&2; return 1;
  }
  yq -o=json '[... | select(tag == "!!map") | keys]' "$1" |
    jq -se 'length == 1 and all(.[0][]; length == (unique | length))' >/dev/null || {
      echo "actions-lock: malformed YAML, multiple documents or duplicate mapping keys: $1" >&2; return 1;
    }
  yq -o=json '.' "$1" | jq -sce 'if length == 1 then .[0] else error("expected one YAML document") end'
}
# Only actual job/step fields count. Ordinary env/input maps, strings and
# block scalars can contain the word uses without declaring a dependency.
lock_uses() {
  lock_yaml_json "$1" | jq -r '
    def emit($kind):
      if has("uses") then .uses |
        if type != "string" or length == 0 or test("[\t\r\n]") then error("uses must be a nonempty single-line string")
        else [$kind, .] | @tsv end
      else empty end;
    if type != "object" then error("workflow/action must be a mapping") else
      (.jobs // {} | .[] | emit("job")),
      (.jobs // {} | .[] | .steps // [] | .[] | emit("step")),
      (.runs.steps? // [] | .[] | emit("step"))
    end'
}
local_action_paths() {
  lock_uses "$1" | awk -F'\t' 'substr($2, 1, 2) == "./" { print $1 "\t" ($2 == "./" ? "." : substr($2, 3)) }'
}
reach_files() {
  local -A seen=()
  local queue=("$1") f kind rel a found paths
  while [ "${#queue[@]}" -gt 0 ]; do
    f="${queue[0]}"; queue=("${queue[@]:1}")
    [ -n "${seen[$f]:-}" ] && continue
    seen[$f]=1
    printf '%s\n' "$f"
    paths=$(local_action_paths "$f") || return 1
    while IFS=$'\t' read -r kind rel; do
      [ -n "$rel" ] || continue
      rel="${rel%/}"
      if [ "$kind" = job ]; then
        case "$rel" in
          *.yml|*.yaml)
            [ -f "$LC_ROOT/$rel" ] && continue ;;
        esac
        echo "actions-lock: missing local reusable workflow: $rel" >&2; return 1
      fi
      found=false
      for a in "$LC_ROOT/$rel/action.yml" "$LC_ROOT/$rel/action.yaml"; do
        if [ -f "$a" ]; then queue+=("$a"); found=true; break; fi
      done
      "$found" && continue
      echo "actions-lock: missing local action metadata: $rel" >&2; return 1
    done <<< "$paths"
  done
}
# ── end of the shared block ───────────────────────────────────────────────────

# reach map: "file<TAB>workflow" — each file whose uses: count for that workflow.
TMP="$(mktemp -d)"; trap 'rm -rf -- "$TMP"' EXIT
MAP="$TMP/reach.tsv"
for wf in "${WORKFLOWS[@]}"; do
  reached=$(reach_files "$wf") || exit 1
  while IFS= read -r f; do printf '%s\t%s\n' "$f" "$wf"; done <<< "$reached"
done > "$MAP"
mapfile -t SCAN < <(cut -f1 "$MAP" | LC_ALL=C sort -u)

# Materialize typed YAML fields for the relational comparison below. No YAML
# spelling (block/flow, quoting or comments) can omit a dependency edge.
LOCK_ROWS="$TMP/lock.tsv"
REF_ROWS="$TMP/uses.tsv"
lock_yaml_json "$LOCK" > "$TMP/lock.json" || exit 1
# An exact-SHA ref exemption belongs only to a key actually called as a
# job-level reusable workflow. A leaf or composite uses list is not proof.
: > "$TMP/callers.txt"
for wf in "${WORKFLOWS[@]}"; do
  lock_uses "$wf" | jq -Rr '
    split("\t") | select(.[0] == "job") | .[1] |
    capture("^(?<owner>[A-Za-z0-9._-]+)/(?<repo>[A-Za-z0-9._-]+)/\\.github/workflows/[^/]+\\.ya?ml@(?<ref>.+)$")? |
    (.owner + "/" + .repo + "@" + .ref | ascii_downcase)
  ' >> "$TMP/callers.txt"
done
jq -Rsc 'split("\n") | map(select(length > 0)) | unique' "$TMP/callers.txt" > "$TMP/callers.json"
jq -e --slurpfile callers "$TMP/callers.json" '
  def coordinate: type == "string" and test("^[A-Za-z0-9._-]+/[A-Za-z0-9._-]+@[^:]+$");
  def full_sha: test("^([0-9a-fA-F]{40}|[0-9a-fA-F]{64})$");
  def recordable_ref:
    type == "string" and length > 0 and . != "@" and
    (startswith("-") or startswith("/") or startswith("refs/") or endswith("/") or endswith(".") or contains("..") or contains("@{") or contains("//") or full_sha or test("-g[0-9a-fA-F]{4,}$") | not) and
    (explode | all(.[]; . as $c | $c >= 32 and $c != 127 and ([32,126,94,58,63,42,91,92] | index($c) | not))) and
    all(split("/")[]; (startswith(".") or endswith(".lock") | not));
  def matches_ref($key):
    ($key | index("@")) as $at | $key[$at + 1:] as $pin |
    if ($pin | full_sha) then
      (.commit | ascii_downcase) == ("sha1-" + ($pin | ascii_downcase)) and
      ((.ref | recordable_ref) or
       (.ref == $pin and ($callers[0] | index($key | ascii_downcase)) != null))
    else .ref == $pin and (.ref | recordable_ref) end;
  def positive_id: type == "number" and . > 0 and floor == .;
  def only_keys($allowed): (keys - $allowed | length) == 0;
  # These estate locks run on github.com; foreign GHE records cannot execute
  # here even when their hostname has the right schema-level spelling.
  def hostname: type == "string" and . == "github.com";
  .version as $schema |
  type == "object" and only_keys(["version", "workflows", "dependencies"]) and
  ($schema == "v0.0.2" or $schema == "v0.0.3") and
  (.workflows | type == "object") and (.dependencies | type == "object") and
  all(.workflows | to_entries[];
    (.key | test("^\\.github/workflows/[^/]+\\.ya?ml$")) and
    (.value | type == "array" and all(.[]; coordinate))) and
  all(.dependencies | to_entries[];
    (.key | coordinate) and
    (.key as $key | .value | type == "object" and
      only_keys(["ref", "commit", "owner_id", "repo_id", "uses"] + (if $schema == "v0.0.3" then ["hostname"] else [] end)) and
      (.ref | type == "string" and length > 0) and
      (.commit | type == "string" and test("^sha1-[0-9a-fA-F]{40}$")) and
      (.owner_id | positive_id) and (.repo_id | positive_id) and
      (if has("hostname") then .hostname | hostname else true end) and
      matches_ref($key) and
      (if has("uses") then .uses | type == "array" and all(.[]; coordinate) else true end)))
' "$TMP/lock.json" >/dev/null || { echo 'actions-lock: unsupported schema or malformed workflow/dependency records' >&2; exit 1; }
# Keep parent/child relations for cycle detection; membership alone only proves
# closure. Memoized three-state DFS visits each normalized record once.
jq -e '
  def norm: index("@") as $at | (.[:$at] | ascii_downcase) + .[$at:];
  def visit($graph; $node):
    if .[$node] == 1 then error("dependency cycle at " + $node)
    elif .[$node] == 2 then .
    else .[$node] = 1 |
      reduce ($graph[$node] // [])[] as $child (.; visit($graph; $child)) |
      .[$node] = 2
    end;
  (.dependencies | to_entries | map({key:(.key | norm), value:(.value.uses // [] | map(norm))}) | from_entries) as $graph |
  reduce ($graph | keys[]) as $node ({}; visit($graph; $node)) | true
' "$TMP/lock.json" >/dev/null || { echo 'actions-lock: cyclic dependency graph' >&2; exit 1; }
jq -r '
  (.workflows | to_entries[] | . as $wf |
    (["W", $wf.key, ($wf.value | length == 0)] | @tsv),
    ($wf.value[] | ["L", $wf.key, .] | @tsv)),
  (.dependencies | to_entries[] |
    (["R", .key] | @tsv), (.value.uses[]? | ["E", .] | @tsv))
' "$TMP/lock.json" > "$LOCK_ROWS"
: > "$REF_ROWS"
for f in "${SCAN[@]}"; do
  refs=$(lock_uses "$f") || exit 1
  while IFS= read -r ref; do
    [ -n "$ref" ] && printf '%s\t%s\n' "$f" "$ref"
  done <<< "$refs"
done > "$REF_ROWS"

awk -F'\t' -v lockfile="$LOCK_ROWS" -v refsfile="$REF_ROWS" -v mapfile="$MAP" -v closurefile="$CLOSURE" '
BEGIN {
  # owners[file]: the workflows whose uses: set includes this file. WFS: the
  # workflows themselves, in glob order (a workflow is its own first owner).
  while ((getline l < mapfile) > 0) {
    split(l, mp, "\t")
    owners[mp[1]] = owners[mp[1]] SUBSEP mp[2]
    if (mp[1] == mp[2]) WFS[++nw] = mp[2]
  }
  # THE REVIEWED CLOSURE IS A THIRD SOURCE OF `uses:`. A ref reached THROUGH a
  # remote reusable appears in no workflow text here — it is listed in the
  # closure baseline as "workflow <TAB> reusable <TAB> ref". The
  # unified-dataset-vocab#81 shape that clause 1 deliberately permits (job-level
  # reusable ref absent from the lock, its through-ref recorded under the
  # CALLING workflow) then failed direction 2: the through-ref looked like a
  # stale lock entry because nothing in the YAML names it. Both directions must
  # agree about what "referenced" means, or the supported shape cannot pass this
  # gate at all.
  if (closurefile != "") {
    while ((getline c < closurefile) > 0) {
      if (c ~ /^#/ || c ~ /^[[:space:]]*$/) continue
      split(c, cf, "\t")
      if (cf[1] == "" || cf[3] == "") continue
      reached[cf[1], norm(cf[3])] = 1
    }
    close(closurefile)
  }
}
# owner/repo[/subpath...]@ref  ->  owner/repo@ref   ("" if not an external ref)
function norm(r,   at, path, ref, n, parts) {
  at = index(r, "@")
  if (at == 0) return ""
  path = substr(r, 1, at - 1); ref = substr(r, at + 1)
  if (path == "" || ref == "") return ""
  if (path ~ /^docker:\/\//) return "" # OCI image pins are not GitHub action records.
  if (substr(path, 1, 2) == "./" || substr(path, 1, 2) == "$/") return ""   # local action
  if (split(path, parts, "/") < 2) return ""
  # GitHub owner/repository names are case-insensitive; refs are not.
  return tolower(parts[1] "/" parts[2]) "@" ref
}

# ---------- parsed lock relations ----------
FILENAME == lockfile {
  split($0, row, "\t")
  if (row[1] == "W") { seen_path[row[2]]++; empty_path[row[2]] = row[3] == "true" }
  else if (row[1] == "L") {
    r = norm(row[3]); lock[row[2], r] = 1; want[r] = 1; lockcount[row[2]]++
  } else if (row[1] == "R") {
    r = norm(row[2]); record_seen[r]++
    if (record_seen[r] == 1) haverec[r] = 1
    else { delete haverec[r]; bad_record[r] = 1 }
  } else if (row[1] == "E") want[norm(row[2])] = 1
  next
}
# ---------- parsed job/step uses, attributed to every calling workflow ----------
FILENAME == refsfile {
  split($0, row, "\t")
  nown = split(substr(owners[row[1]], 2), own, SUBSEP)
  raw = row[3]
  if (raw ~ /^\$\//) {
    for (i = 1; i <= nown; i++) dollar[own[i]] = dollar[own[i]] " " raw
    next
  }
  n = norm(raw)
  if (n == "") next
  for (i = 1; i <= nown; i++) {
    uses[own[i], n] = 1
    if (row[2] == "job") joblist[own[i]] = joblist[own[i]] " " n
    else useslist[own[i]] = useslist[own[i]] " " n
  }
  next
}

END {
  bad = 0
  for (key in seen_path) {
    if (seen_path[key] != 1 || (!empty_path[key] && lockcount[key] == 0)) bad_workflow[key] = 1
  }
  for (key in bad_workflow) {
    printf "FAIL %s\n     malformed workflow lock value; expected [] or a reference list\n", key
    bad = 1
  }
  for (w = 1; w <= nw; w++) {
    wf = WFS[w]
    key = wf
    sub(/.*\//, "", key)
    key = ".github/workflows/" key          # the lockfile always uses this canonical path

    if (dollar[wf] != "") {
      printf "FAIL %s\n     invalid local-action rewrite (uses: $/...):%s\n", key, dollar[wf]
      bad = 1
    }

    # --- direction 1: every STEP-LEVEL uses: must be locked under THIS path ---
    #
    # A job-level reusable ref that is absent from the lockfile is HARMLESS.
    # MEASURED (standards#968, 2026-09-22): 159 repositories carry job-level
    # refs the lock does not; 12 of 12 sampled created jobs, and
    # the hyperpolymath/standards mirror.yml has a literally empty entry while
    # calling a reusable and running 7 jobs. Demanding it also steers a
    # developer into the FATAL state, because `gh actions-lock` cannot backfill
    # a job-level ref: the only way to go green was to hand-add a per-workflow
    # entry with no record, which is exactly the dangling edge that kills a run.
    # The reconciliation case is hyperpolymath/unified-dataset-vocab#81: it
    # records the through-ref (actions/checkout) and no reusable record at all,
    # and its caller runs 3 jobs with 10/3/3 steps.
    # A job-level ref that IS recorded is still checked: closure belongs to
    # check-lock-pin-contract.sh (which delegates to close-lock.sh --check), and
    # the dependency set it brings in belongs to check-reusable-closure.sh.
    nu = split(useslist[wf], u, " ")
    delete uniq; missing = ""
    for (j = 1; j <= nu; j++) {
      if (u[j] == "" || (u[j] in uniq)) continue
      uniq[u[j]] = 1
      if (!((key SUBSEP u[j]) in lock)) missing = missing " " u[j]
    }
    if (missing != "") {
      if (!(key in seen_path))
        printf "FAIL %s\n     not onboarded: no lockfile entry for this path\n     unlocked refs:%s\n", key, missing
      else
        printf "FAIL %s\n     refs missing from the lockfile:%s\n", key, missing
      bad = 1
    }

    # job-level refs absent from the lockfile: reported, never fatal.
    nj = split(joblist[wf], v, " ")
    delete jseen
    for (j = 1; j <= nj; j++) {
      if (v[j] == "" || (v[j] in jseen)) continue
      jseen[v[j]] = 1
      if (!((key SUBSEP v[j]) in lock)) jnote = jnote sprintf("\n  %s: %s (allowed — measured harmless, standards#968)", key, v[j])
    }

    # --- direction 2: every lock entry must be referenced by this workflow ---
    orphan = ""
    for (k in lock) {
      split(k, kp, SUBSEP)
      if (kp[1] != key) continue
      if (!((wf SUBSEP kp[2]) in uses) && !((key SUBSEP kp[2]) in reached)) orphan = orphan " " kp[2]
    }
    if (orphan != "") {
      printf "FAIL %s\n     stale lockfile entries, no uses: references them:%s\n", key, orphan
      bad = 1
    }
  }

  # --- lockfile entries for workflow files that no longer exist ---
  for (p in seen_path) {
    found = 0
    for (w = 1; w <= nw; w++) {
      q = WFS[w]
      sub(/.*\//, "", q); q = ".github/workflows/" q
      if (q == p) { found = 1; break }
    }
    if (!found) { printf "FAIL %s\n     lockfile entry for a workflow file that does not exist\n", p; bad = 1 }
  }

  # Every workflow needs a key, even one without actions. Every recorded edge
  # must resolve, including edges introduced by dependency records themselves.
  for (w = 1; w <= nw; w++) {
    key = WFS[w]; sub(/.*\//, "", key); key = ".github/workflows/" key
    if (!(key in seen_path)) {
      printf "FAIL %s\n     workflow has no lockfile key (use [] when it has no actions)\n", key
      bad = 1
    }
  }
  for (r in want) {
    if (!(r in haverec)) {
      printf "FAIL actions.lock\n     dangling dependency edge: %s\n", r
      bad = 1
    }
  }
  for (r in haverec) {
    if (!(r in want)) {
      printf "FAIL actions.lock\n     unused dependency record: %s\n", r
      bad = 1
    }
  }
  for (r in bad_record) {
    printf "FAIL actions.lock\n     ambiguous dependency record coordinate: %s\n", r
    bad = 1
  }
  if (jnote != "") print "NOTE: job-level reusable refs absent from the lockfile (not fatal; see standards#968):" jnote

  if (bad) {
    print ""
    print "actions.lock is OUT OF SYNC with the workflow YAML."
    print "GitHub refuses such a run at startup: zero jobs are created and the run"
    print "reports \"This run likely failed because of a workflow file issue.\""
    print "Fix: run `gh actions-lock --no-migrate-local-actions`, then review the diff"
    print "(it does not handle job-level reusable-workflow refs, and it can de-pin"
    print "bare SHAs to floating tags - both must be corrected by hand)."
    exit 1
  }
  print "actions.lock is in sync: every uses: is locked under its own workflow path,"
  print "and every lockfile entry is referenced. Job-level reusable-workflow refs and"
  print "refs used through local composite actions included."
}
' "$LOCK_ROWS" "$REF_ROWS"
