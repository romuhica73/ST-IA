# ST-IA Community

This document explains what the public ST-IA repository is, what it contains,
and how it relates to distributions that may be offered separately later.

*Ce document est en anglais, comme le README. Le reste de la documentation du
projet est en français.*

---

## What this repository is

**ST-IA Community is the complete ST-IA application, published under the MIT
license.**

It is not a demo, a trial, or a feature-reduced edition. If you clone this
repository and build it, you get the product: French transcription, English
translation, SRT and TXT output, model management, cancellation and recovery —
the whole thing.

What it asks of you is the ability to build it yourself. There is no installer
here, and no signed binary. That is the actual distinction between Community
and anything that may come later — packaging and convenience, not capability.

## What is included

| Capability | Status in Community |
|---|---|
| Local French transcription (whisper.cpp, Metal) | included |
| Local French → English translation | included |
| SRT and TXT output — FR, EN, or both | included |
| On-demand model download, pinned and checksum-verified | included |
| Real progress, cancellation, recovery after failure | included |
| Disclosure of which AI models run, and where | included |
| Full v0.1 user interface, FR/EN, light/dark, reduced motion | included |
| Ready-to-install signed installer | **not included** — build from source |

## Local-first, and what that actually means

ST-IA processes media on your machine. There is no server, no account, and no
telemetry.

It is **not** true that ST-IA never uses the network. It makes exactly one kind
of network request: downloading a Whisper model, when you explicitly click to
do so. That request fetches a pinned file from Hugging Face and sends no media,
no transcript and no filename. Once the model is on disk, the application works
entirely offline — you can turn off Wi-Fi and verify this.

Details, including model checksums and provenance: [`AI_MODELS.md`](AI_MODELS.md).

## License

ST-IA's own code is [MIT](../LICENSE).

Third-party components shipped alongside it keep their own licenses — FFmpeg
(LGPL-2.1, shipped as a separate executable) and whisper.cpp (MIT). See
[`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md).

Whisper model weights are **not** redistributed by this repository. They are
downloaded from Hugging Face at your explicit request.

## Building from source

macOS on Apple Silicon (arm64) is currently the only qualified platform.
Windows is not supported yet — see
[`platforms/WINDOWS_PORT_PLAN.md`](platforms/WINDOWS_PORT_PLAN.md).

Full instructions: [`BUILDING.md`](BUILDING.md).

## Contributing

Contributions are welcome, within a deliberately narrow scope. Read
[`CONTRIBUTING.md`](../CONTRIBUTING.md) first — particularly the three
non-negotiable rules (local-first, no telemetry, no cloud transcription), which
determine whether a contribution can be merged at all.

Vulnerabilities go through [`SECURITY.md`](../SECURITY.md), never through a
public issue.

## Relationship to future distributions

Official ready-to-use distributions may be offered separately in the future.
Nothing of the sort exists today: there is no release, no installer, no pricing,
and no subscription.

The boundary between this repository and any such distribution is decided in
[ADR-012](architecture/ADR-012-community-commercial-boundary.md). The part worth
knowing as a user or contributor:

* the core capabilities listed above stay in Community and will not be removed
  from it in order to be sold;
* proprietary features are never developed inside this MIT repository — not
  even disabled, not even behind a flag;
* anything commercial would live in a separate private repository that consumes
  this one as upstream.

## Versioning

Community follows standard SemVer: `v0.1.0`, `v0.2.0`, and so on.

**`v0.1.0` is the first tagged version** — a **source release**. It publishes
the code, not a binary: there is no signed installer, no notarized `.app` and
no `.dmg` attached to it. Building from source is how you run it.
