#!/usr/bin/env python3
"""Light structural validation of the SRT files ST-IA produces.

Deliberately not a full SRT parser: it checks the properties that would
actually break a downstream editor like DaVinci Resolve — decodable as UTF-8,
contiguous numbering, well-formed and non-decreasing timestamps, and no empty
cue text.

Usage: scripts/validate-srt.py FILE [FILE...]
Exit code 0 if every file passes, 1 otherwise.
"""

import re
import sys

TIMING = re.compile(
    r"^(\d{2}):(\d{2}):(\d{2}),(\d{3}) --> (\d{2}):(\d{2}):(\d{2}),(\d{3})$"
)


def to_ms(h, m, s, ms):
    return ((int(h) * 60 + int(m)) * 60 + int(s)) * 1000 + int(ms)


def validate(path):
    errors = []
    try:
        with open(path, "rb") as fh:
            raw = fh.read()
    except OSError as exc:
        return [f"cannot read: {exc}"], 0

    if not raw:
        return ["file is empty"], 0

    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as exc:
        return [f"not valid UTF-8: {exc}"], 0

    # Blank-line separated blocks; tolerate CRLF and a trailing newline.
    blocks = [b for b in re.split(r"\r?\n\r?\n", text.strip()) if b.strip()]
    if not blocks:
        return ["no subtitle blocks found"], 0

    previous_end = -1
    for position, block in enumerate(blocks, start=1):
        lines = block.splitlines()
        if len(lines) < 3:
            errors.append(f"block {position}: expected index, timing and text")
            continue

        if lines[0].strip() != str(position):
            errors.append(
                f"block {position}: index is {lines[0].strip()!r}, expected {position}"
            )

        match = TIMING.match(lines[1].strip())
        if not match:
            errors.append(f"block {position}: malformed timing {lines[1].strip()!r}")
            continue

        start = to_ms(*match.group(1, 2, 3, 4))
        end = to_ms(*match.group(5, 6, 7, 8))
        if end < start:
            errors.append(f"block {position}: end precedes start")
        if start < previous_end:
            errors.append(f"block {position}: start goes backwards")
        previous_end = end

        if not "".join(lines[2:]).strip():
            errors.append(f"block {position}: empty cue text")

    return errors, len(blocks)


def main(paths):
    failed = False
    for path in paths:
        errors, count = validate(path)
        if errors:
            failed = True
            print(f"FAIL {path} ({count} blocks)")
            for error in errors[:10]:
                print(f"  - {error}")
            if len(errors) > 10:
                print(f"  … and {len(errors) - 10} more")
        else:
            print(f"OK   {path} ({count} blocks)")
    return 1 if failed else 0


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(2)
    sys.exit(main(sys.argv[1:]))
