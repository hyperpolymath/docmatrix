#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
set -euo pipefail
repo=$(cd -- "$(dirname -- "$0")/.." && pwd)
fixture=$(mktemp -d)
trap 'rm -rf -- "$fixture"' EXIT
wf="$fixture/repo/.github/workflows"
mkdir -p "$wf"
passed=0
edit_file() {
  local expression=$1 file=$2
  sed -E "$expression" "$file" > "$fixture/edited"
  mv "$fixture/edited" "$file"
}
expect() {
  local wanted=$1 label=$2 status=0
  bash "$repo/scripts/check-lock-sync.sh" "$wf" > "$fixture/output" 2>&1 || status=$?
  if [[ "$status" != "$wanted" ]]; then
    cat "$fixture/output" >&2
    printf 'FAIL %s: expected %s, got %s\n' "$label" "$wanted" "$status" >&2
    exit 1
  fi
  passed=$((passed + 1))
  printf 'PASS %s\n' "$label"
}
action_workflow() {
  cat > "$wf/probe.yml" <<'YAML'
name: probe
on: push
jobs:
  test:
    runs-on: ubuntu-24.04
    steps:
      - uses: example/action@v1
YAML
}
good_lock() {
  cat > "$wf/actions.lock" <<'LOCK'
version: 'v0.0.3'
workflows:
    '.github/workflows/probe.yml':
        - 'example/action@v1'
dependencies:
    'example/action@v1':
        ref: 'v1'
        commit: 'sha1-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
        owner_id: 1
        repo_id: 2
LOCK
}
action_workflow
good_lock
expect 0 'complete action lock'
sed '/^dependencies:/,$d' "$wf/actions.lock" > "$fixture/cut"
mv "$fixture/cut" "$wf/actions.lock"
printf 'dependencies: {}\n' >> "$wf/actions.lock"
expect 1 'missing direct dependency record'
good_lock
cat >> "$wf/actions.lock" <<'LOCK'
        uses:
            - 'example/helper@v2'
LOCK
expect 1 'missing transitive dependency record'
cat >> "$wf/actions.lock" <<'LOCK'
    'example/helper@v2':
        ref: 'v2'
        commit: 'sha1-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'
        owner_id: 1
        repo_id: 3
LOCK
expect 0 'complete transitive dependency record'
good_lock
printf 'name: empty\non: push\njobs: {}\n' > "$wf/empty.yml"
expect 1 'unlisted zero-action workflow'
rm -- "$wf/empty.yml"
cat > "$wf/probe.yml" <<'YAML'
name: probe
on: push
jobs:
  child:
    uses: example/action/.github/workflows/child.yml@v1
YAML
cat > "$wf/actions.lock" <<'LOCK'
version: 'v0.0.3'
workflows:
    '.github/workflows/probe.yml': []
dependencies: {}
LOCK
expect 0 'omitted reusable call uses its callee-owned lock'
cat >> "$wf/probe.yml" <<'YAML'
  inline:
    runs-on: ubuntu-24.04
    steps:
      - uses: example/action@v1
YAML
expect 1 'reusable call cannot hide an unlocked step at the same coordinate'
action_workflow
good_lock
mkdir -p "$fixture/repo/.github/actions/local"
cat > "$wf/probe.yml" <<'YAML'
name: probe
on: push
jobs:
  test:
    runs-on: ubuntu-24.04
    steps:
      - uses: ./.github/actions/local
YAML
cat > "$fixture/repo/.github/actions/local/action.yml" <<'YAML'
name: local
runs:
  using: composite
  steps:
    - uses: example/action@v1
YAML
expect 0 'local composite dependency remains locked under its caller'
sed '/^dependencies:/,$d' "$wf/actions.lock" > "$fixture/cut"
mv "$fixture/cut" "$wf/actions.lock"
printf 'dependencies: {}\n' >> "$wf/actions.lock"
expect 1 'local composite cannot hide a missing dependency record'
cat > "$wf/actions.lock" <<'LOCK'
version: 'v0.0.3'
workflows:
    '.github/workflows/probe.yml': []
dependencies: {}
LOCK
rm "$fixture/repo/.github/actions/local/action.yml"
expect 1 'missing local action metadata is fatal'
cat > "$wf/probe.yml" <<'YAML'
jobs:
  child:
    uses: ./.github/workflows/missing.yml
YAML
expect 1 'missing local reusable workflow is fatal'
for scalar in '|' '>-'; do
  cat > "$wf/probe.yml" <<YAML
jobs:
  test:
    steps:
      - run: $scalar
          echo 'sample generated YAML'
          uses: ./.github/actions/example-only
YAML
  expect 0 "sample local use in block scalar is ignored: $scalar"
  printf '      - uses: ./.github/actions/actually-missing\n' >> "$wf/probe.yml"
  expect 1 "real local use after block scalar is enforced: $scalar"
done
printf 'jobs: {}\n' > "$wf/probe.yml"
for value in null '' false '[null]'; do
  printf "version: 'v0.0.3'\nworkflows:\n    '.github/workflows/probe.yml': %s\ndependencies: {}\n" "$value" > "$wf/actions.lock"
  expect 1 "malformed workflow value is refused: $value"
done
printf "version: 'v0.0.3'\nworkflows:\n    '.github/workflows/probe.yml': []\ndependencies: {}\n" > "$wf/actions.lock"
for suffix in yml yaml; do
  mkdir -p "$fixture/repo/.github/actions/valid.$suffix"
  printf 'runs:\n  using: composite\n  steps: []\n' > "$fixture/repo/.github/actions/valid.$suffix/action.yml"
  printf 'jobs:\n  test:\n    steps:\n      - uses: ./.github/actions/valid.%s\n' "$suffix" > "$wf/probe.yml"
  expect 0 "local action directory may end in .$suffix"
done
action_workflow
for field in ref commit owner_id repo_id; do
  good_lock
  edit_file "/^        $field:/d" "$wf/actions.lock"
  expect 1 "missing required record field: $field"
done
good_lock
edit_file "s/^    'example\/action@v1':$/    'example\/action@v1': null/; /^        (ref|commit|owner_id|repo_id):/d" "$wf/actions.lock"
expect 1 'null dependency record'
good_lock
edit_file "s/^        commit:.*/        commit: 'sha1-short'/" "$wf/actions.lock"
expect 1 'malformed commit digest'
good_lock
edit_file "s/^        owner_id:.*/        owner_id: '1'/" "$wf/actions.lock"
expect 1 'quoted owner id is not an integer'
good_lock
edit_file "/^        ref:/p" "$wf/actions.lock"
expect 1 'duplicate required record field'
good_lock
edit_file 's/example\/action@v1/example\/action@Release@2026/g' "$wf/probe.yml"
edit_file 's/example\/action@v1/example\/action@Release@2026/g' "$wf/actions.lock"
edit_file "s/ref: 'v1'/ref: 'Release@2026'/" "$wf/actions.lock"
expect 0 'embedded at-sign preserves the complete ref'
edit_file "s/^    'example\/action@Release@2026':/    'example\/action@release@2026':/" "$wf/actions.lock"
expect 1 'embedded at-sign does not hide a ref-case mismatch'
action_workflow
good_lock
edit_file '/^        (ref|commit|owner_id|repo_id):/s/$/ # provenance/' "$wf/actions.lock"
expect 0 'inline comments on required fields are accepted'
edit_file 's/example\/action@v1/example\/action@v#1/g' "$wf/probe.yml"
good_lock
edit_file "s/example\/action@v1/example\/action@v#1/g; s/ref: 'v1'/ref: 'v#1' # literal hash/" "$wf/actions.lock"
expect 0 'hash inside a quoted ref is preserved'
action_workflow
for value in "'''" "'unclosed" '"v1"junk' '"\q"' '"\U00110000"' '"\uD800"'; do
  good_lock
  edit_file '/^        ref:/d' "$wf/actions.lock"
  printf '        ref: %s\n' "$value" >> "$wf/actions.lock"
  expect 1 "malformed quoted scalar is refused: $value"
done
good_lock
edit_file '/^        ref:/d' "$wf/actions.lock"
printf '        ref: "%s"\n' '\u0076\u0031' >> "$wf/actions.lock"
expect 0 'valid escaped Unicode ref scalar is accepted'
good_lock
printf 'dependencies: {}\n' >> "$wf/actions.lock"
expect 1 'duplicate dependency section is refused'
good_lock
printf 'workflows: {}\n' >> "$wf/actions.lock"
expect 1 'duplicate workflow section is refused'
good_lock
printf "        uses: ['example/helper@v2']\n" >> "$wf/actions.lock"
expect 1 'flow-style transitive dependency cannot omit its record'
cat >> "$wf/actions.lock" <<'LOCK'
    'example/helper@v2':
        ref: 'v2'
        commit: 'sha1-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'
        owner_id: 1
        repo_id: 3
LOCK
expect 0 'flow-style transitive dependency with its record passes'
good_lock
cat >> "$wf/probe.yml" <<'YAML'
        with:
          uses: ./ordinary-input-data
env:
  uses: ./ordinary-environment-data
YAML
expect 0 'ordinary uses keys in environment and action inputs are ignored'
printf "version: 'v0.0.3'\nworkflows:\n    '.github/workflows/probe.yml': []\ndependencies: {}\n" > "$wf/actions.lock"
for scalar in '|' '>-'; do
  cat > "$wf/probe.yml" <<YAML
jobs:
  test:
    steps:
      - run: $scalar
          echo 'sample YAML'
          uses: example/not-a-dependency@v1
YAML
  expect 0 "sample external use in block scalar is ignored: $scalar"
done
action_workflow
good_lock
edit_file "s/^    'example\/action@v1':/    'example\/action\/subdir@v1':/" "$wf/actions.lock"
expect 1 'dependency record keys cannot include action subpaths'
good_lock
edit_file 's/uses: example\/action@v1/uses: example\/action\/subdir@v1/' "$wf/probe.yml"
expect 0 'action uses may include a subpath with a repository-owned record'
action_workflow
good_lock
cat >> "$wf/actions.lock" <<'LOCK'
    'example/act!on@v1':
        ref: 'v1'
        commit: 'sha1-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
        owner_id: 1
        repo_id: 2
LOCK
expect 1 'dependency record repository names use coordinate syntax'
good_lock
printf 'jobs:\n  test:\n    steps:\n      - uses: ./\n' > "$wf/probe.yml"
printf 'runs:\n  using: composite\n  steps:\n    - uses: example/action@v1\n' > "$fixture/repo/action.yml"
expect 0 'root local composite contributes its remote references'
sed '/^dependencies:/,$d' "$wf/actions.lock" > "$fixture/cut"
mv "$fixture/cut" "$wf/actions.lock"
printf 'dependencies: {}\n' >> "$wf/actions.lock"
expect 1 'root composite cannot hide a missing remote dependency record'
rm "$fixture/repo/action.yml"
printf "version: 'v0.0.3'\nworkflows:\n    '.github/workflows/probe.yml': []\ndependencies: {}\n" > "$wf/actions.lock"
expect 1 'root local action without metadata is refused'
action_workflow
good_lock
edit_file '/^version:/d' "$wf/actions.lock"
expect 1 'missing lock schema version is refused'
good_lock
edit_file "s/v0.0.3/v9.9.9/" "$wf/actions.lock"
expect 1 'unsupported lock schema version is refused'
good_lock
edit_file "s/v0.0.3/v0.0.2/" "$wf/actions.lock"
expect 0 'the reviewed v0.0.2 schema remains supported'
good_lock
printf 'unexpected: true\n' >> "$wf/actions.lock"
expect 1 'unknown root properties are refused'
good_lock
printf '        unexpected: true\n' >> "$wf/actions.lock"
expect 1 'unknown action properties are refused'
for host in github.com; do
  good_lock
  printf "        hostname: '%s'\n" "$host" >> "$wf/actions.lock"
  expect 0 "canonical v0.0.3 hostname is accepted: $host"
done
for host in acme.ghe.com HTTPS://github.com/ Github.com -acme.ghe.com acme-.ghe.com ' github.com ' "$(printf '%064d' 0).ghe.com"; do
  good_lock
  printf "        hostname: '%s'\n" "$host" >> "$wf/actions.lock"
  expect 1 "noncanonical hostname is refused: $host"
done
good_lock
edit_file 's/v0.0.3/v0.0.2/' "$wf/actions.lock"
printf "        hostname: 'github.com'\n" >> "$wf/actions.lock"
expect 1 'v0.0.2 does not admit the v0.0.3 hostname field'
good_lock
cat >> "$wf/actions.lock" <<'LOCK'
    'example/helper@v:2':
        ref: 'v:2'
        commit: 'sha1-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'
        owner_id: 1
        repo_id: 3
LOCK
expect 1 'colon in an unused canonical dependency pin is refused'
printf 'version: v0.0.3\nworkflows:\n    '\''.github/workflows/probe.yml'\'': []\ndependencies: {}\n' > "$wf/actions.lock"
printf 'jobs:\n  test:\n    steps:\n      - uses: docker://alpine@sha256:%s\n' "$(printf 'a%.0s' {1..64})" > "$wf/probe.yml"
expect 0 'digest-pinned OCI actions do not require GitHub repository records'
action_workflow
good_lock
edit_file "s/ref: 'v1'/ref: 'v2'/" "$wf/actions.lock"
expect 1 'a symbolic record ref must match its coordinate'
for bad in 'main~1' 'v1^' 'refs/tags/v1' 'v1-gabcd' 'bad.lock' 'v1@{1}'; do
  action_workflow; good_lock
  edit_file "s#example/action@v1#example/action@$bad#g; s#ref: 'v1'#ref: '$bad'#" "$wf/actions.lock"
  edit_file "s#example/action@v1#example/action@$bad#g" "$wf/probe.yml"
  expect 1 "unrecordable ref name is refused: $bad"
done
action_workflow; good_lock
pin=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
edit_file "s/example\/action@v1/example\/action@$pin/g" "$wf/actions.lock"
edit_file "s/example\/action@v1/example\/action@$pin/g" "$wf/probe.yml"
expect 1 'SHA coordinates must match the recorded commit'
edit_file "s/sha1-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/sha1-$pin/" "$wf/actions.lock"
expect 0 'SHA coordinate may record a matching commit and symbolic tag'
edit_file "s/ref: 'v1'/ref: '$pin'/" "$wf/actions.lock"
expect 1 'an ordinary leaf action cannot repeat its pin as ref'
printf '        uses: []\n' >> "$wf/actions.lock"
expect 1 'a composite uses list alone cannot authorize a raw-SHA ref'
printf '  child:\n    uses: EXAMPLE/ACTION/.github/workflows/child.yml@%s\n' "$pin" >> "$wf/probe.yml"
expect 0 'proven job-level caller-side SHA-ref representation remains supported'
edit_file "s/ref: '$pin'/ref: 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'/" "$wf/actions.lock"
expect 1 'a SHA record cannot name an unrelated SHA as its ref'
action_workflow; good_lock
printf "        uses: ['example/action@v1']\n" >> "$wf/actions.lock"
expect 1 'self dependency cycle is refused'
good_lock
cat >> "$wf/actions.lock" <<'LOCK'
        uses: ['example/helper@v2']
    'example/helper@v2':
        ref: 'v2'
        commit: 'sha1-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'
        owner_id: 1
        repo_id: 3
        uses: ['EXAMPLE/ACTION@v1']
LOCK
expect 1 'multi-record cycles normalize repository case'
good_lock
cat >> "$wf/actions.lock" <<'LOCK'
        uses: ['example/left@v1', 'example/right@v1']
    'example/left@v1':
        ref: 'v1'
        commit: 'sha1-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'
        owner_id: 1
        repo_id: 3
        uses: ['example/shared@v1']
    'example/right@v1':
        ref: 'v1'
        commit: 'sha1-cccccccccccccccccccccccccccccccccccccccc'
        owner_id: 1
        repo_id: 4
        uses: ['example/shared@v1']
    'example/shared@v1':
        ref: 'v1'
        commit: 'sha1-dddddddddddddddddddddddddddddddddddddddd'
        owner_id: 1
        repo_id: 5
LOCK
expect 0 'a shared acyclic child is not mistaken for a cycle'
good_lock
cat >> "$wf/actions.lock" <<'LOCK'
        uses: ['example/helper/subdir@v2']
    'example/helper@v2':
        ref: 'v2'
        commit: 'sha1-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'
        owner_id: 1
        repo_id: 3
LOCK
expect 1 'transitive pins must be repository coordinates'
cat > "$wf/probe.yml" <<'YAML'
jobs:
  test:
    steps:
      - uses: "example/action@release'2026"
YAML
cat > "$wf/actions.lock" <<'LOCK'
version: v0.0.3
workflows:
    '.github/workflows/probe.yml':
        - 'example/action@release''2026'
dependencies:
    'example/action@release''2026':
        ref: 'release''2026'
        commit: 'sha1-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
        owner_id: 1
        repo_id: 2
LOCK
expect 0 'valid apostrophes in ref names remain supported'
action_workflow; good_lock
cat >> "$wf/actions.lock" <<'LOCK'
    'example/orphan@v1':
        ref: 'v1'
        commit: 'sha1-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'
        owner_id: 1
        repo_id: 3
LOCK
expect 1 'unused dependency records are refused'
good_lock
cat >> "$wf/actions.lock" <<'LOCK'
    'example/orphan@v1':
        ref: 'v1'
        commit: 'sha1-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'
        owner_id: 1
        repo_id: 3
        uses: ['example/child@v1']
    'example/child@v1':
        ref: 'v1'
        commit: 'sha1-cccccccccccccccccccccccccccccccccccccccc'
        owner_id: 1
        repo_id: 4
LOCK
expect 1 'a disconnected acyclic dependency tree is refused at its root'
mkdir -p "$fixture/repo/.github/actions"
printf 'jobs: {}\n' > "$fixture/repo/.github/actions/plain.yml"
printf 'jobs: {test: {steps: [{uses: ./.github/actions/plain.yml}]}}\n' > "$wf/probe.yml"
printf 'version: v0.0.3\nworkflows: {.github/workflows/probe.yml: []}\ndependencies: {}\n' > "$wf/actions.lock"
expect 1 'a regular YAML file cannot stand in for step action metadata'
printf 'jobs: {}\n' > "$wf/callee.yml"
printf 'jobs: {child: {uses: ./.github/workflows/callee.yml}}\n' > "$wf/probe.yml"
printf 'version: v0.0.3\nworkflows: {.github/workflows/probe.yml: [], .github/workflows/callee.yml: []}\ndependencies: {}\n' > "$wf/actions.lock"
expect 0 'an existing local reusable workflow is accepted at job level'
printf 'jobs: {test: {steps: [{uses: ./.github/workflows/callee.yml}]}}\n' > "$wf/probe.yml"
expect 1 'the same local workflow file is refused as a step action'
printf 'passed %s controls\n' "$passed"
