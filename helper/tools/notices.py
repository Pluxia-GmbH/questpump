#!/usr/bin/env python3
"""The licence texts of every Rust crate the desktop app builds in (docs/plan.md slice 35, the Acknowledgements on the
computer): `cargo metadata --locked` gives Cargo.lock's graph for the targets QuestPump ships on (macOS and Windows,
both architectures), questpump-helper's normal dependencies are followed (a dev or build dependency never ships, nor do
the workspace's own crates), and each crate's licence and notice files (LICENSE*, LICENCE*, COPYING*, NOTICE* in its
manifest folder, or the file its manifest names) are written to helper/notices/crates.txt, which the app builds in
(desk/menu_pages/about.rs) and its Acknowledgements show. A text several crates share is written once under the crates
it covers: a licence's terms (the Apache License 2.0 through "END OF TERMS AND CONDITIONS", left without its appendix
when that is the blank template; a permission notice from "Permission is hereby granted") once, each crate's own lines
around them (its copyright, a filled-in appendix) under its name. A crate without a licence file of its own is listed
with its licence expression and its source.

    python3 helper/tools/notices.py [--offline]           writes helper/notices/crates.txt
    python3 helper/tools/notices.py --check [--offline]   fails when helper/notices/crates.txt is not what it would
                                                          write (the helper's local gates, rust-helper step 2)

--offline reads only the crates' sources Cargo has on this computer already. Standard library only."""
import json
import os
import re
import subprocess
import sys

HELPER = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(HELPER, "notices", "crates.txt")
APP = "questpump-helper"
# the macOS app on both architectures and Windows x64 (what helper-release.yml builds) and Windows arm64 (what
# tools/windows-check.sh lints; helper.yml built its installer until 2026-10-09)
TARGETS = ("aarch64-apple-darwin", "x86_64-apple-darwin", "x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc")
FILES = re.compile(r"(licen[cs]e|copying|notice)", re.IGNORECASE)
# between the texts; the lines that are this file's own are not wrapped, so the Acknowledgements' window wraps them
RULE = "=" * 40
# a licence's terms several crates share, by the line that opens them and the line that closes them (none: the end)
TERMS = (
    ("TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION", "END OF TERMS AND CONDITIONS"),
    ("Permission is hereby granted, free of charge", None),
)
# the Apache License's title lines, which belong to its terms above the line that opens them
TITLE = re.compile(r"apache license|version 2\.0, january 2004|apache\.org/licenses/(license-2\.0)?$", re.IGNORECASE)
# the Apache License's appendix, how to apply it to a work, and its boilerplate notice left blank, which names no one
APPENDIX = "APPENDIX: How to apply the Apache License"
TEMPLATE = re.compile(r"\[yyyy\]|\{yyyy\}|\[name of copyright owner\]", re.IGNORECASE)
TEMPLATE_END = "limitations under the License"
HEAD = " ".join((
    "QuestPump on this computer builds in these Rust crates (Cargo.lock; the macOS and Windows apps), each under its",
    "licence. Each text is here once, after the crates it covers and the lines of their own (a copyright, a notice).",
    "Written by helper/tools/notices.py: do not edit.",
))


def metadata(offline):
    """Cargo.lock's graph for the shipped targets."""
    command = ["cargo", "metadata", "--format-version", "1", "--locked"]
    command += ["--offline"] if offline else []
    for target in TARGETS:
        command += ["--filter-platform", target]
    done = subprocess.run(command, cwd=HELPER, capture_output=True, text=True)
    if done.returncode:
        sys.exit(f"cargo metadata failed:\n{done.stderr}")
    return json.loads(done.stdout)


def shipped(meta):
    """The packages the app builds in: questpump-helper's normal dependencies, all the way down, by name and version."""
    packages = {p["id"]: p for p in meta["packages"]}
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
    members = set(meta["workspace_members"])
    app = next(i for i in members if packages[i]["name"] == APP)
    seen, todo = set(), [app]
    while todo:
        node = todo.pop()
        if node in seen:
            continue
        seen.add(node)
        todo += [d["pkg"] for d in nodes[node]["deps"] if any(k["kind"] is None for k in d["dep_kinds"])]
    return sorted((packages[i] for i in seen - members), key=lambda p: (p["name"], version_key(p["version"])))


def version_key(version):
    """A version in its numeric order (1.10 after 1.9)."""
    return [int(part) if part.isdigit() else part for part in re.split(r"[.+-]", version)]


def read(path):
    """A file's text with its lines as written, the ends of lines and of the file trimmed."""
    with open(path, "rb") as fh:
        raw = fh.read()
    try:
        text = raw.decode("utf-8-sig")
    except UnicodeDecodeError:
        text = raw.decode("latin-1")
    lines = [line.rstrip() for line in text.replace("\r\n", "\n").replace("\r", "\n").split("\n")]
    return "\n".join(lines).strip("\n")


def files(package):
    """A package's licence and notice files, in their names' order."""
    folder = os.path.dirname(package["manifest_path"])
    paths = [os.path.join(folder, name) for name in sorted(os.listdir(folder)) if FILES.match(name)]
    paths = [path for path in paths if os.path.isfile(path)]
    named = package.get("license_file")
    if named:
        path = os.path.normpath(os.path.join(folder, named))
        if os.path.isfile(path) and path not in paths:
            paths.append(path)
    return paths


def collapsed(text):
    """A text as the grouping compares it: its words, whatever the wrapping and the indent."""
    return " ".join(text.split())


def trimmed(lines):
    """Lines without blank lines at either end, as one text."""
    return "\n".join(lines).strip("\n")


def own_lines(lines):
    """The lines around a licence's terms, as a crate's own: the Apache License's appendix left out (its instructions,
    and its boilerplate notice while it is the blank template), a boilerplate notice filled in kept."""
    at = next((i for i, line in enumerate(lines) if APPENDIX in line), None)
    if at is not None:
        copyright = (i for i in range(at, len(lines)) if lines[i].strip().lower().startswith("copyright"))
        notice = next(copyright, len(lines))
        end = notice
        if notice < len(lines) and TEMPLATE.search(lines[notice]):
            end = next((i + 1 for i in range(notice, len(lines)) if TEMPLATE_END in lines[i]), notice + 1)
        lines = lines[:at] + lines[end:]
    return trimmed(lines)


def split(text):
    """A file's text as (the terms several crates share, or the whole text; the key they are grouped by; the lines of
    its own, or none). The key starts at the line that opens the terms, so a title written another way groups alike."""
    lines = text.split("\n")
    for opening, closing in TERMS:
        start = next((i for i, line in enumerate(lines) if opening in line), None)
        if start is None:
            continue
        end = len(lines)
        if closing:
            end = next((i + 1 for i in range(start, len(lines)) if closing in lines[i]), None)
            if end is None:
                continue
        key = collapsed("\n".join(lines[start:end]))
        # the title lines above the terms are the terms' own
        while start > 0 and (not lines[start - 1].strip() or TITLE.search(lines[start - 1].strip())):
            start -= 1
        while start < end and not lines[start].strip():
            start += 1
        own = "\n\n".join(part for part in (own_lines(lines[:start]), own_lines(lines[end:])) if part)
        return trimmed(lines[start:end]), key, own or None
    return text, collapsed(text), None


def name(package):
    return f"{package['name']} {package['version']}"


def listed(label, names):
    """A list of crates on one line."""
    return f"{label} {', '.join(names)}"


def render(packages):
    """crates.txt: each text with the crates it covers, the texts most crates share first; then the crates without a
    licence file."""
    groups = {}  # terms' key -> {"text", "crates", "own": {collapsed own lines -> (text, [crates])}}
    bare = []
    for package in packages:
        paths = files(package)
        if not paths:
            bare.append(package)
            continue
        for path in paths:
            terms, key, own = split(read(path))
            if not terms:
                continue
            group = groups.setdefault(key, {"text": terms, "crates": [], "own": {}})
            if name(package) not in group["crates"]:
                group["crates"].append(name(package))
            if own:
                entry = group["own"].setdefault(collapsed(own), (own, []))
                if name(package) not in entry[1]:
                    entry[1].append(name(package))
    out = [HEAD]
    for group in sorted(groups.values(), key=lambda g: (-len(g["crates"]), g["crates"][0], g["text"])):
        part = [RULE, listed(f"{len(group['crates'])} {'crate' if len(group['crates']) == 1 else 'crates'}:",
                             group["crates"])]
        for own, crates in sorted(group["own"].values(), key=lambda entry: (entry[1][0], entry[0])):
            part.append(f"{listed('--', crates)}\n{own}")
        part.append(group["text"])
        out.append("\n\n".join(part))
    if bare:
        lines = [RULE, "Crates without a licence file of their own, with their licence (Cargo.toml) and source:"]
        for package in bare:
            source = package.get("repository") or package.get("homepage") or "crates.io"
            lines.append(f"{name(package)}: {package.get('license') or 'no licence named'}, {source}")
        out.append("\n".join(lines))
    return "\n\n".join(out) + "\n"


def main(argv):
    unknown = [arg for arg in argv if arg not in ("--check", "--offline")]
    if unknown:
        sys.exit(f"notices.py: unknown arguments {' '.join(unknown)}; it takes --check and --offline")
    text = render(shipped(metadata("--offline" in argv)))
    rel = os.path.relpath(OUT, os.path.dirname(HELPER))
    if "--check" in argv:
        try:
            with open(OUT, encoding="utf-8") as fh:
                current = fh.read()
        except FileNotFoundError:
            current = None
        if current != text:
            sys.exit(f"{rel} is stale: run python3 helper/tools/notices.py and commit it")
        return
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as fh:
        fh.write(text)
    print(f"wrote {rel} ({len(text.encode('utf-8'))} bytes)")


if __name__ == "__main__":
    main(sys.argv[1:])
