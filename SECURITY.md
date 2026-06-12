# Security Policy

## Reporting a Vulnerability

Report vulnerabilities privately via
[GitHub Security Advisories](https://github.com/zircote/rlm-rs/security/advisories/new).
Do not open a public issue for security reports.

## Verifying Release Artifacts

Every release binary is built on GitHub Actions and carries
[SLSA build provenance](https://slsa.dev/spec/v1.0/provenance) attested with
`actions/attest-build-provenance`. The release pipeline verifies every
attestation fail-closed before the GitHub Release is published — a tag
publishes nothing unattested.

To verify a downloaded artifact yourself (requires the
[`gh` CLI](https://cli.github.com/), authenticated):

```sh
gh release download v<version> --repo zircote/rlm-rs
gh attestation verify rlm-cli-<version>-<platform> --repo zircote/rlm-rs
```

For example:

```sh
gh attestation verify rlm-cli-1.3.0-linux-amd64 --repo zircote/rlm-rs
```

A successful verification prints `✓ Verification succeeded!` and confirms
the binary is byte-identical to what GitHub Actions built from this
repository. Verification fails closed if the file was modified, rebuilt
elsewhere, or attested by any other repository or workflow.

### SBOM

Each release ships a CycloneDX SBOM (`rlm-cli-<version>-sbom.cdx.json`)
generated with Syft, and every binary carries an SBOM attestation binding
it to that SBOM. To verify:

```sh
gh attestation verify rlm-cli-<version>-<platform> --repo zircote/rlm-rs \
  --predicate-type https://cyclonedx.org/bom
```

### Checksums

`rlm-cli-<version>-checksums.txt` lists SHA-256 digests of every release
asset for quick integrity checks (`sha256sum -c`). Checksums are a
convenience; the attestations above are the authoritative, fail-closed
verification path.
