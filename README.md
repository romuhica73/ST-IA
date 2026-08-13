# ST-IA

**Turn an audio or video file into subtitles, entirely on your own Mac.**

ST-IA is a local-first macOS desktop application. You give it a media file, it
gives you `.srt` and `.txt` — a French transcription, an English translation, or
both. Nothing is uploaded: the speech recognition runs on your machine.

```text
your media  →  FFmpeg (local)  →  whisper.cpp (local, Metal)  →  SRT + TXT
```

> **Status: `v0.1.0` — source release.** No official binary distribution is
> published: there is no signed installer and no `.dmg` to download. You build
> ST-IA from source — see [Build from source](#build-from-source).

*Documentation note: this README and [`docs/COMMUNITY_EDITION.md`](docs/COMMUNITY_EDITION.md)
are in English. The rest of the project documentation is in French.*

---

## Why ST-IA

* **Local-first.** Transcription and translation happen on your Mac. Your media
  never leaves it.
* **No cloud media processing.** There is no inference endpoint in the code to
  call, because the engine is an embedded executable.
* **No account, no telemetry.** Nothing to sign up for, nothing reported back.
* **French transcription** with a Whisper `large-v3-turbo` model.
* **English translation** produced locally by a second, dedicated model — not by
  an online service.
* **Honest about the network.** ST-IA makes exactly one kind of network request:
  downloading a model when you explicitly ask it to. See [Privacy](#privacy).

## Platform status

| Platform | Status |
|---|---|
| **macOS, Apple Silicon (arm64)** | **Qualified** — the only tested and supported target |
| macOS, Intel | Not supported |
| Windows | **Not yet supported** — [port plan](docs/platforms/WINDOWS_PORT_PLAN.md), no date announced |
| Linux | Not supported, not planned |

The committed sidecar binaries are arm64 Mach-O executables and the acceleration
backend is Metal. Open source does not mean cross-platform.

## Quick start

For a technical user on an Apple Silicon Mac. Full detail:
[`docs/QUICKSTART.md`](docs/QUICKSTART.md).

```sh
git clone https://github.com/romuhica73/ST-IA.git
cd ST-IA
pnpm install --frozen-lockfile
pnpm tauri build
```

Then open `src-tauri/target/release/bundle/macos/ST-IA.app`.

The build is **not signed or notarized**, so Gatekeeper will block it on first
launch: right-click → **Open** → confirm. This is expected for a local build.

On first run, ST-IA asks you to download the transcription model. Point it at a
media file, choose which versions you want (French, English, or both), and the
`.srt` and `.txt` files are written next to your media.

## Build from source

Prerequisites, reference tool versions, tests, sidecar rebuilds and
troubleshooting: **[`docs/BUILDING.md`](docs/BUILDING.md)**.

Requirements in short: macOS on Apple Silicon, Xcode Command Line Tools,
Node.js 20 LTS, pnpm 10, Rust 1.96. No Homebrew package is needed at runtime —
both sidecars are static and link only Apple system frameworks.

## Models

**No model ships with the application.** ST-IA downloads them on your explicit
click, verifies them, and stores them under
`~/Library/Application Support/com.romainbourbon.stia/models/`.

| Model | Role | Size | Required |
|---|---|---|---|
| `ggml-large-v3-turbo-q5_0.bin` | French transcription | 574 MB | always |
| `ggml-large-v3.bin` | French → English translation | 3.1 GB | only if you ask for English output |

Both are fetched from a **pinned commit** of `huggingface.co/ggerganov/whisper.cpp`
— never a branch pointer — and are rejected unless their exact size and SHA-256
match the pinned manifest.

Provenance, checksums, the engine that runs them, and the known limitations
(technical vocabulary, translation repetition) are documented factually in
**[`docs/AI_MODELS.md`](docs/AI_MODELS.md)**. That document is a disclosure, not
a claim of regulatory conformity.

## Privacy

Stated precisely, because a vague privacy claim is worth nothing:

* **Your media, transcripts and file paths never leave your machine.** They are
  read locally, processed by local executables, and written back to disk next to
  your media.
* **There is no account, no telemetry, no analytics, no crash reporting and no
  auto-update.**
* **ST-IA does use the network — for one thing only:** downloading a Whisper
  model, triggered by an explicit click. That request is a plain GET for a
  pinned file. It carries no media, no transcript and no filename.
* After the model is downloaded, the application runs fully offline. Turn off
  Wi-Fi and check.
* Preferences are stored locally in `Application Support` and are never sent
  anywhere.

The full threat model — assets, trust boundaries, what is actually guaranteed
and which risks are knowingly accepted — is in
[`docs/security/THREAT_MODEL.md`](docs/security/THREAT_MODEL.md).

## Security

Please report vulnerabilities through GitHub's private
["Report a vulnerability"](https://github.com/romuhica73/ST-IA/security/advisories/new)
form — **never in a public issue**. Scope, expectations and response times:
[`SECURITY.md`](SECURITY.md).

Reviews: [M8 security review](docs/security/M8_SECURITY_REVIEW.md),
[M9 delta](docs/security/M9_SECURITY_DELTA.md),
[M10 public release review](docs/security/M10_COMMUNITY_PUBLIC_SECURITY_REVIEW.md).

## Contributing

Contributions are welcome within a deliberately narrow scope. Read
[`CONTRIBUTING.md`](CONTRIBUTING.md) first — especially the three
non-negotiable rules (local-first, no telemetry, no cloud transcription), which
decide whether a change can be merged at all.

## License

ST-IA's own code is [MIT](LICENSE).

Third-party components shipped with the application keep their own licenses:

* **FFmpeg** — LGPL-2.1, shipped as a separate executable
  ([details](docs/third-party/FFMPEG.md));
* **whisper.cpp** — MIT.

See [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md). Whisper model weights are
not redistributed here — they are downloaded from Hugging Face at your request.

## This repository, and what may come later

This is **ST-IA Community**: the complete application, MIT, for people able to
build it themselves. It is not a trial or a cut-down edition — the full
transcription and translation pipeline is here.

Official ready-to-use distributions may be offered separately in the future.
Nothing of the kind exists today — no release, no installer, no pricing. The
boundary is documented in
[`docs/COMMUNITY_EDITION.md`](docs/COMMUNITY_EDITION.md) and
[ADR-012](docs/architecture/ADR-012-community-commercial-boundary.md).

---

## Documentation

**Getting started**

* [Quick start](docs/QUICKSTART.md)
* [Build from source](docs/BUILDING.md)
* [AI models, provenance and limitations](docs/AI_MODELS.md)
* [ST-IA Community — what this repository is](docs/COMMUNITY_EDITION.md)

**Project**

* [Roadmap](docs/ROADMAP.md)
* [Changelog](CHANGELOG.md)
* [Contributing](CONTRIBUTING.md)
* [Security policy](SECURITY.md)
* [Windows port plan](docs/platforms/WINDOWS_PORT_PLAN.md)

**Architecture decisions** ([index](docs/architecture/README.md))

| ADR | Decision |
|---|---|
| [001](docs/architecture/ADR-001-transcription-engine.md) | Local transcription engine |
| [002](docs/architecture/ADR-002-desktop-architecture.md) | Desktop architecture |
| [003](docs/architecture/ADR-003-local-transcription-pipeline.md) | Local pipeline and engine packaging |
| [004](docs/architecture/ADR-004-model-management.md) | Model management and integrity |
| [005](docs/architecture/ADR-005-runtime-lifecycle-and-cancellation.md) | Job lifecycle, cancellation, cleanup |
| [006](docs/architecture/ADR-006-release-identity-and-data-migration.md) | Release identity and data migration |
| [007](docs/architecture/ADR-007-local-preferences-and-interface-localization.md) | Local preferences and localization |
| [008](docs/architecture/ADR-008-bilingual-output-pipeline.md) | Bilingual output — the turbo model finding |
| [009](docs/architecture/ADR-009-splashscreen-and-release-packaging.md) | Integrated splash and release packaging |
| [010](docs/architecture/ADR-010-local-english-translation.md) | Local English translation |
| [011](docs/architecture/ADR-011-fixed-desktop-shell.md) | Fixed desktop shell and motion system |
| [012](docs/architecture/ADR-012-community-commercial-boundary.md) | Community / Desktop / Plus boundary |

**Security and licensing**

* [Threat model](docs/security/THREAT_MODEL.md)
* [M8 security review](docs/security/M8_SECURITY_REVIEW.md)
* [M9 security delta](docs/security/M9_SECURITY_DELTA.md)
* [M10 public release review](docs/security/M10_COMMUNITY_PUBLIC_SECURITY_REVIEW.md)
* [Project license (MIT)](LICENSE)
* [Third-party components and licenses](THIRD_PARTY_NOTICES.md)
* [FFmpeg — sidecar and license](docs/third-party/FFMPEG.md)
