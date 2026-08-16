# M3 — Desktop distribution state

**Version audited** 0.1.0 (tag `v0.1.0`) · **Branch** `feat/somyuren-m3-desktop-distribution`

What follows is what the repository actually contains, established by reading
the source rather than by reading the marketing site.

## Product inventory — what exists

| Area | State |
|---|---|
| Media selection / drop zone | Real (`src/features/media-selection`) |
| Transcription (French) | Real (`src/features/transcription`) |
| Translation (English) | Real |
| Exports SRT / TXT | Real |
| Model manager | Real (`src/features/model-manager`, `src-tauri/src/model.rs`) |
| Settings / About | Real (`src/features/settings`) |
| Boot splash | Real (`src/features/boot`) |
| **History** | **Does not exist** |
| **Batch queue** | **Does not exist** |

The decisive line is in `src-tauri/src/pipeline.rs`:

> *"Single in-memory job slot — supports exactly one transcription at a time,
> per mission scope (no job queue, no database)."*

A history list needs a database; a batch queue needs a queue. Neither exists.

### Consequence for the Premium contract

The website advertises **"Traitement par lots" as P1 of a future Premium
edition, available 2027-01-01**, and states it is absent from the current
version. **That is accurate.** There is no batch queue in the product, so
`BATCH-PREMIUM-PRODUCT-DECISION-GATE` is **not required** — the conflict
suspected during M2 was an artefact of a mockup image, not of the product.

### Consequence for the website

M2 published four images from `docs/screenshots/` (an **untracked** directory)
as "captures de l'application". Two of that set depict the non-existent history
and batch screens, and one of those was published. All of them show SOMYUREN
branding, which the application's own UI does not yet have. They are design
mockups. The website has been corrected and the slots returned to placeholder.

## Network behaviour — audited

Every outbound URL in the Rust source:

| URL | Purpose |
|---|---|
| `huggingface.co/.../ggml-large-v3-turbo-q5_0.bin` | model download |
| `huggingface.co/.../ggml-large-v3.bin` | model download |

Both are pinned to an exact upstream commit (`5359861c…`) rather than a moving
`main`. Models are verified by **size and SHA-256** before use, and a mismatch
yields a `Corrupted` state rather than silent acceptance.

**No telemetry, no analytics, no crash reporting** — searched for and absent.
The privacy claims the website makes about the product are, on this evidence,
true.

## Identity

| | Before | After |
|---|---|---|
| Product name | `ST-IA` | **`SOMYUREN`** |
| Bundle identifier | `com.romainbourbon.stia` | **`com.somyuren.desktop`** |
| UI strings | "ST-IA analyse toujours l'audio" | SOMYUREN |

The identifier moves to the domain the project actually controls. **No binary
has ever been distributed**, so no installed user is affected — this is the
last moment the change is free.

Application data is protected by extending the existing migration
(`src-tauri/src/migration.rs`) from a single legacy identifier to an ordered
**chain**:

```
com.somyuren.desktop        ← current
com.romainbourbon.stia      ← M5 release candidate
com.romainbourbon.st-ia.dev ← M0–M4 development
```

The chain is searched newest-first and the first *valid* model is adopted, so a
user who skipped a release still keeps their 574 MB download. The migration
remains deliberately narrow: one file, by exact name, verified by hash, with
foreign files left untouched. Covered by 7 tests.

## Platform truth

| Platform | State | Evidence |
|---|---|---|
| macOS Apple Silicon | **Primary target** | Sidecars are `aarch64-apple-darwin`; builds locally |
| macOS Intel | **Not supported** | No `x86_64-apple-darwin` sidecars |
| Windows x64 | **Not buildable today** | **No Windows sidecars exist** |
| Windows ARM | Not targeted | — |
| Linux | Not targeted | — |

### Why Windows is not merely "unfinished"

`tauri.conf.json` requires two external binaries:

```
binaries/whisper-cli
binaries/ffmpeg
```

Only `-aarch64-apple-darwin` variants exist. Windows needs
`whisper-cli-x86_64-pc-windows-msvc.exe` and its ffmpeg counterpart, and the
scripts that produce them (`scripts/build-*-sidecar.sh`) are bash targeting
macOS. Producing governed, checksummed Windows sidecars is a substantial piece
of work in its own right — not a flag on a build command.

Encouragingly, the Rust source contains **no `cfg(target_os)` branches**, so
there is no macOS-specific logic to port; the blocker is the native
dependencies, not the application.

## Gates

| Gate | State | Closure condition |
|---|---|---|
| `APPLE_DISTRIBUTION_SIGNING_GATE` | **OPEN** | A Developer ID Application certificate. `security find-identity -v -p codesigning` reports **0 valid identities**; no notarytool profile exists. Without it there is no signed, notarised, Gatekeeper-clean build, and therefore no public macOS beta. |
| `WINDOWS_CODE_SIGNING_GATE` | **OPEN (blocked behind the build)** | Moot until Windows sidecars exist. |
| `BETA-PUBLICATION-GATE` | **OPEN** | Human approval, and at minimum a signed macOS artefact. |
| `BATCH-PREMIUM-PRODUCT-DECISION-GATE` | **NOT REQUIRED** | No batch queue exists; no conflict. |
| `DESKTOP-BUNDLE-ID-GATE` | **CLOSED** | Human approved `com.somyuren.desktop`; nothing shipped under the old identifier. |

## What an unsigned artefact is good for

An unsigned `.app`/`.dmg` is a legitimate **internal QA artefact** and nothing
more. macOS Gatekeeper will refuse it, and the remedy — instructing users to
right-click-Open or run `xattr` — is not an installation procedure a product
may ship. Any artefact produced before the Apple gate closes is therefore
marked **NOT FOR PUBLIC DISTRIBUTION**.
