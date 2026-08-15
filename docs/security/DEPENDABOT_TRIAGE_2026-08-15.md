# Dependabot triage — 2026-08-15

A security follow-up mission on the private Desktop distribution
(`romuhica73/ST-IA-Desktop`) flagged two dependency advisories and
recorded `COMMUNITY_SECURITY_FOLLOWUP_REQUIRED`, since Desktop's
`Cargo.lock`/`pnpm-lock.yaml` are descended from this repository and the
same entries could plausibly exist here too. This document is that
follow-up, analyzed independently from this repository's own state
rather than assumed from Desktop's conclusions.

GitHub's own Dependabot alerts API reported zero open alerts for this
repository at the time of this triage, despite `dependabot_security_updates`
being enabled. That is a visibility gap, not evidence of a clean
dependency tree — both `pnpm audit` and direct lockfile inspection below
confirm one of the two advisories is genuinely present here.

---

## nanoid (npm, HIGH)

| | |
|---|---|
| GHSA | [GHSA-2v37-7h3g-55p8](https://github.com/advisories/GHSA-2v37-7h3g-55p8) |
| CVE | CVE-2026-67213 |
| Package | `nanoid` |
| Vulnerable | `< 3.3.18` |
| Fixed | `>= 3.3.18` |
| Manifest | `pnpm-lock.yaml` |

**Present**: yes — confirmed directly in `pnpm-lock.yaml`
(`nanoid@3.3.17`) and via `pnpm audit`, independently of GitHub's
Dependabot UI.

**Dependency position**: `TRANSITIVE_DEV`. `package.json`'s own
`dependencies` are `@tauri-apps/api`, `@tauri-apps/plugin-dialog`,
`i18next`, `react`, `react-dom`, `react-i18next` — none reach `nanoid`.
`pnpm why nanoid` traces the only path:

```
nanoid@3.3.17
└─┬ postcss@8.5.26
  └─┬ vite@7.3.6
    ├── st-ia@0.1.0 (devDependencies)
    └── (vitest, @vitejs/plugin-react — all devDependencies)
```

**Shipped?** No. `pnpm audit --prod` was clean both before and after this
fix. The production build (`pnpm build`) output is a static `dist/`
bundle that never references `nanoid`; the bundle hashes changed only
because of the normal chunk-hashing that any dependency-tree change
produces, not because `nanoid` itself entered the bundle.

**Reachability**: `NOT_REACHABLE_IN_SHIPPED_PRODUCT`. The advisory
requires an application to pass an unvalidated, attacker-controlled
`size: 0` to `customAlphabet`/`customRandom`. `postcss`'s internal use is
a fixed, non-configurable call with no external input — no path exists,
in the shipped app or in the build pipeline, for attacker-controlled
input to reach it.

**End-user risk**: none — not present in the `.app` bundle.

**Developer/CI risk**: negligible — exercises only `vite`/`vitest`
processing this repository's own CSS/JS sources during local development
or CI, never untrusted media, filenames, or transcript content.

**Fix**: `pnpm update nanoid` → resolves to `3.3.18` within `postcss`'s
already-declared `^3.3.17` range. Four lines changed in
`pnpm-lock.yaml` (the version pin and its snapshot entry, each appearing
twice); no other package touched. `pnpm install --frozen-lockfile`
confirms the resulting lockfile is internally consistent. `pnpm audit` /
`pnpm audit --prod`: clean after the fix.

**Residual risk**: none.

**Status**: `FIXED`.

---

## glib (cargo, MODERATE)

| | |
|---|---|
| GHSA | [GHSA-wrw7-89jp-8q8g](https://github.com/advisories/GHSA-wrw7-89jp-8q8g) |
| Package | `glib` |
| Vulnerable | `>= 0.15.0, < 0.20.0` |
| Fixed | `>= 0.20.0` |
| Manifest | `src-tauri/Cargo.lock` |
| Resolved version | `0.18.5` |

**Present**: yes, in `Cargo.lock` — same version as Desktop's (`0.18.5`),
via the identical chain: `tauri → gtk → atk → glib`.

**Community's own supported platform** — checked directly, not assumed
from Desktop's classification: [`CONTRIBUTING.md`](../../CONTRIBUTING.md)
states plainly, *"macOS Apple Silicon (arm64) uniquement. C'est la seule
plateforme qualifiée."* — "Intel macOS, Windows and Linux are neither
tested, nor built, nor supported." [`docs/BUILDING.md`](../BUILDING.md) is
equally explicit: *"Open source ne veut pas dire multiplateforme. Il n'y
a **aucun** build Windows ou Linux."* Community does not claim, build, or
test for Linux — if anything its portability promise is narrower than
Desktop's (which at least builds and ships for Windows).

**Shipped? Proven no**:

1. `cargo tree -i glib` against the real build target
   (`aarch64-apple-darwin`) returns **"nothing to print."**
2. Tauri's own `Cargo.toml` gates the entire GTK stack (`gtk`,
   `webkit2gtk`) to
   `cfg(any(target_os = "linux", "dragonfly", "freebsd", "openbsd", "netbsd"))`
   — `macos` is not in that list, and it is the only platform this
   repository builds for.

`glib` is therefore never compiled, never linked, and cannot exist in
the `.app` this repository produces, under any build configuration this
project actually uses.

**Reachability**: `NOT_REACHABLE_IN_SHIPPED_PRODUCT`, `PLATFORM_SPECIFIC`
— exercised only on a Linux/BSD Tauri target this repository neither
builds nor supports.

**Attempted fix**:

```
$ cargo update -p glib --precise 0.20.0 --dry-run
error: failed to select a version for the requirement `glib = "^0.18"`
candidate versions found which didn't match: 0.20.0
required by package `gtk v0.18.2`
    ... which satisfies dependency `gtk = "^0.18"` (locked to 0.18.2) of
        package `tauri v2.11.5`
```

Identical constraint to Desktop's finding: `gtk v0.18.2` — pinned by
`tauri v2.11.5` itself, not by anything in this repository — requires
`glib = "^0.18"`, incompatible with `0.20.0`. No version of `glib` alone
can be pinned here without breaking `gtk`'s own declared constraint; this
is an upstream Tauri/`gtk-rs` matter.

**Since Community explicitly does not support Linux, this is not a
`COMMUNITY_LINUX_SECURITY_FOLLOWUP_REQUIRED` situation** — that
classification would apply if Community claimed or built for Linux while
carrying an unfixable Linux vulnerability. It does not.

**Residual risk**: none, for the actual shipped/supported platform.

**Follow-up target**: same as Desktop's — re-check on the next routine
`cargo update` after a Tauri release bumps its `gtk`/`gtk-rs` dependency
range to `glib >= 0.20`. No dedicated mission is warranted for a
platform-gated, never-compiled dependency on a platform this project
does not build for.

**Status**: `OPEN — DOCUMENTED, NOT REACHABLE, NO SAFE FIX AVAILABLE`.

---

## Other advisories

No other HIGH or CRITICAL advisory was found during this audit — `pnpm
audit` reported exactly the one finding above before remediation, and
`cargo audit` was not available in this environment (not installed
locally; this repository's own CI security workflow, GitHub-hosted, is
the intended source of that evidence going forward).

---

## Summary

| Advisory | Package | Severity | Shipped | Status |
|---|---|---|---|---|
| GHSA-2v37-7h3g-55p8 | nanoid | HIGH | No (dev-only) | Fixed — `3.3.17` → `3.3.18` |
| GHSA-wrw7-89jp-8q8g | glib | MODERATE | No (platform-gated, macOS-only build) | Open — documented, no safe fix available upstream |

Neither advisory represented a reachable risk to the shipped `.app`.
`nanoid` is fully remediated. `glib` cannot be remediated from this
repository without an upstream Tauri release, and poses no actual risk
given Community's own, explicitly macOS-only build.
