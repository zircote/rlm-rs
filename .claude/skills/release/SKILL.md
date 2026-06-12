---
name: release
argument-hint: v<X.Y.Z> | patch | minor | major
description: >-
  Orchestrate and monitor a full attested release of rlm-rs end-to-end:
  release-prep PR, tag, attested binaries + SBOM, crates.io Trusted
  Publishing with .crate attestation, Homebrew propagation, and independent
  workstation verification. Use this skill whenever the user invokes
  /release v<n.n.n> or /release patch|minor|major, or says "cut a release",
  "ship a release", "release version X", "bump and release",
  "do a patch/minor/major release", "tag a new version", or
  anything else that means publishing a new version of this project. Do not
  improvise the release process from memory — this skill encodes hard-won
  fixes for failure modes that are invisible until a release breaks.
---

# rlm-rs Release Orchestration

Run a complete attested release for this repository. The argument is either
an explicit version (`/release v1.4.0`) or a bump type
(`/release patch|minor|major`), which computes the target from the current
`Cargo.toml` version. Every phase below ends with a verification; do not
proceed past a failure — fix it or stop and report.

The pipeline this skill drives (all already wired in `.github/workflows/`):

```
prep PR ──merge──> tag push ──┬─> release.yml  (test + audit gates → 4 platform
                              │    binaries → provenance + SBOM attestations →
                              │    fail-closed verify → GitHub Release)
                              └─> publish.yml  (pre-publish checks → crates.io
                                   Trusted Publishing → download registry .crate
                                   → sha256 match → attest)
release.yml completion ─workflow_run─> package-homebrew.yml (formula update)
```

## Help / no argument

If invoked with no argument, `--help`, or `help`, print this and stop —
do not start a release:

```
/release — attested release orchestration for rlm-rs

USAGE
    /release v<X.Y.Z>     release an explicit version (e.g. /release v1.4.0)
    /release patch        bump 1.3.1 -> 1.3.2 and release
    /release minor        bump 1.3.1 -> 1.4.0 and release
    /release major        bump 1.3.1 -> 2.0.0 and release

WHAT IT DOES
    prep PR (5 version locations) -> required-checks green -> squash merge
    -> annotated tag -> monitors: attested binaries + SBOM + fail-closed
    verify -> GitHub Release; crates.io Trusted Publishing + .crate
    attestation; Homebrew auto-update -> independent workstation
    verification of every artifact.

NOTES
    - Publishing to crates.io is irreversible; versions are immutable.
    - CHANGELOG [Unreleased] must have content; empty means stop and ask.
    - Never re-runs release.yml against an existing tag (asset immutability).
```

## Phase 0 — Preflight

1. `git checkout main && git pull`; working tree must be clean of tracked
   changes (untracked noise is fine).
2. Resolve the target version from the argument:
   - `v<major>.<minor>.<patch>` → use as given.
   - `patch` | `minor` | `major` → read `version` from `Cargo.toml` on
     freshly-pulled main and bump that component, zeroing the lower ones
     (`1.3.1` + `minor` → `1.4.0`; `1.3.1` + `major` → `2.0.0`).
   - Anything else → stop and ask.
   Strip the `v` for file edits; keep it for the tag. State the resolved
   version in the first progress message so a wrong bump is caught early.
3. Sanity checks, all hard stops:
   - New version is greater than `version` in `Cargo.toml`.
   - Tag does not already exist (`git tag -l v<X.Y.Z>`, and check remote).
   - `CHANGELOG.md` has content under `## [Unreleased]` — a release with an
     empty changelog means something is off; ask the user what this release
     contains.
   - Latest CI run on main is green
     (`gh run list --workflow=ci.yml --branch main --limit 1`).
4. Semver gut-check: if Unreleased contains breaking changes or an MSRV
   bump and the user asked for a patch, raise it before proceeding.

## Phase 1 — Release prep

Branch `release/v<X.Y.Z>` off main, then update **all five** version
locations (missing one ships inconsistent metadata):

| File | What to change |
| --- | --- |
| `Cargo.toml` | `version = "<X.Y.Z>"` |
| `Cargo.lock` | run `cargo check` after the Toml edit — never hand-edit |
| `CHANGELOG.md` | insert `## [<X.Y.Z>] - <today>` under `## [Unreleased]`; update the `[Unreleased]:` compare link and add the new version's compare link at the bottom |
| `CITATION.cff` | `version:` and `date-released:` — **both occurrences** (top level and `preferred-citation`) |
| `SECURITY.md` | the `rlm-cli-<version>-linux-amd64` example version |

Validate locally before the PR: `cargo fmt -- --check` and `cargo check`
minimum. The PR's CI and the release workflow's own gates run the full
suite, so don't duplicate the entire gauntlet here — but a broken lockfile
or fmt failure should never reach the PR.

Commit as `chore(release): v<X.Y.Z>`, push, open the PR. The body should
list what the release contains and note anything operator-relevant.

## Phase 2 — PR through merge

Required status checks on main are `All Checks Pass` and
`pin-check / pin-check`. Monitor the PR with the Monitor tool — and use
the aggregate-gate guard, not a naive all-non-pending check:

```bash
# Terminal only when the "All Checks Pass" check itself has reported and
# nothing is pending. Right after a push there is a window where only 1-2
# checks are registered; "zero pending" alone declares victory in that
# window (this produced a false ALL GREEN once).
gate=$(jq -r '[.[] | select(.name=="All Checks Pass")][0].bucket // "absent"' <<<"$checks")
```

When green, merge with `gh pr merge --squash --delete-branch`, then
`git checkout main && git pull` and confirm HEAD is the release commit.

## Phase 3 — Tag

Annotated tag on the merge commit, then push:

```bash
git tag -a v<X.Y.Z> -m "Release v<X.Y.Z>

<one-paragraph summary from the changelog>" <merge-sha>
git push origin v<X.Y.Z>
```

The tag push is the release trigger. Facts that matter here:

- Tag pushes bypass branch protection — release.yml carries its own test
  and audit gates precisely because the tag is untrusted input.
- **Never re-dispatch release.yml against an existing tag.** Builds are
  not reproducible; it would overwrite published release assets with
  different bytes, violating the immutability the attestations exist to
  protect.

## Phase 4 — Monitor the chains

Three things run; watch all of them with the Monitor tool (one monitor,
multiple conditions — report each as it lands):

1. **Release run** (`release.yml`, ~10 jobs). Expect: Resolve Version,
   Test, Cargo Audit, 4 × Build, SBOM (generate + attest), Verify
   Attestations, Create Release — all success.
2. **Publish run** (`publish.yml`). Expect: pre-publish checks, Trusted
   Publishing auth, `cargo publish`, then the crate-attestation steps
   ("Download published crate from registry" and "Attest crate
   provenance"). Report these step conclusions explicitly — they are the
   newest links in the chain.
3. **Homebrew run** (`package-homebrew.yml`) must appear **on its own**
   after the Release run completes, via `workflow_run`. If no run appears
   within a few minutes of Release success, the trigger regressed — fall
   back to manual dispatch
   (`gh workflow run package-homebrew.yml -f version=<X.Y.Z> -f dry_run=false`)
   and investigate.

### Failure playbook

| Symptom | Cause | Action |
| --- | --- | --- |
| Publish auth fails: "No Trusted Publishing config found" | crates.io TP not configured | One-time setup on crates.io: rlm-cli → Settings → Trusted Publishing → repo `zircote/rlm-rs`, workflow filename `publish.yml`, environment `copilot`. Then `gh workflow run publish.yml --ref v<X.Y.Z>` (dispatch-on-tag sets `github.ref` to `refs/tags/v<X.Y.Z>`, so the `startsWith(github.ref, 'refs/tags/v')` gates pass and the tag-gated steps run). |
| Publish fails: "crate rlm-cli@X.Y.Z already exists" | Duplicate publish attempt raced a successful one | Benign. Verify the version is live (`cargo search rlm-cli`), report, move on. crates.io versions are immutable. |
| Crate download step exhausts retries | static.crates.io CDN propagation | Re-run the failed job; the publish itself succeeded. |
| Crate sha256 mismatch (registry vs local package) | Should never happen — cargo packaging is deterministic per commit | Hard stop. Do not attest. Investigate before anything else. |
| Cargo Audit job fails | Real advisory in `Cargo.lock` | Fix the dependency (usually `cargo update <crate>`) via a normal PR, then start the release over at Phase 0. Note: cargo-deny may NOT have flagged it — deny analyzes the feature/target graph, audit scans the raw lockfile; an unreachable phantom lock entry trips audit only. Both gates are intentional; keep both. |
| A build leg fails | Platform/toolchain issue | macos-amd64 is intentionally absent (ort ships no x86_64-apple-darwin onnxruntime). The four legs are linux-amd64, linux-arm64 (`ubuntu-24.04-arm`), macos-arm64, windows-amd64. Binaries build with **default features** (matches `cargo install`), not the Makefile's `--all-features`. |
| Release event didn't trigger Homebrew (pre-workflow_run behavior) | Releases are authored by `github-actions[bot]`; the tap PAT is scoped to homebrew-tap only and bot events don't trigger workflows | The `workflow_run` trigger handles this; `head_branch` in the workflow_run payload IS the tag name for tag-triggered runs (verified empirically — and the payload has no `ref` field, whatever a reviewer may claim). |

## Phase 5 — Independent workstation verification

In-pipeline success is necessary; this is the acceptance test. Run from
the local machine, in a scratch dir:

```bash
gh release download v<X.Y.Z> --repo zircote/rlm-rs
# Expect 6 assets: 4 binaries, rlm-cli-<X.Y.Z>-sbom.cdx.json, checksums.txt

for f in rlm-cli-<X.Y.Z>-{linux-amd64,linux-arm64,macos-arm64,windows-amd64.exe}; do
  gh attestation verify "$f" --repo zircote/rlm-rs                  # provenance
  gh attestation verify "$f" --repo zircote/rlm-rs \
    --predicate-type https://cyclonedx.org/bom                      # SBOM binding
done
shasum -a 256 -c rlm-cli-<X.Y.Z>-checksums.txt

# crates.io: needs a User-Agent or the API/CDN rejects silently
curl -fsSL -A 'rlm-rs-release-check' \
  -O https://static.crates.io/crates/rlm-cli/rlm-cli-<X.Y.Z>.crate
gh attestation verify rlm-cli-<X.Y.Z>.crate --repo zircote/rlm-rs
```

Check **exit codes**, not grepped output — a filtered pipe that matches
nothing looks identical to success (this mistake was made once; silence is
not success). No `--signer-workflow` flag anywhere: artifacts here are
attested by this repo's own workflows, not a central signer.

Confirm crates.io shows the version:
`curl -s -A 'rlm-rs-release-check' https://crates.io/api/v1/crates/rlm-cli | jq .crate.max_version`

## Final report

Summarize for the user: version, merge commit, tag; per-channel status
(GitHub Release / crates.io / Homebrew); workstation verification results;
and anything from the failure playbook that fired. If any channel is
incomplete, say exactly what is pending and what unblocks it.
