"""The add-on's media packs (docs/plan.md slice 71): the voice pack QuestPump_Voice and the figure pack
QuestPump_FigArmor, each a load-on-demand folder of its own beside QuestPump in the game, a zip of its own and a store
project of its own, so a release that changes code alone sends no media again. QuestPump plays without either: a move
without its figure shows its key pose, a line without its take stays unsaid.

PACKS is the one list of them: each pack's folder in the zip, where its files sit in the repository (the voice's takes
stay in Media/Voice/, where the tools and the desktop app read them, and ship as the pack's Takes/), what stays out, and
the words its store upload carries as its changelog.

The packager (BigWigsMods/packager's release.sh) zips one checkout into one zip, so each pack is staged as a small Git
repository of its own, which the packager then packs and uploads as it does the add-on (`release.sh -t <dir>`, the
store's project by its -p or -a):

    python3 -B tools/addon_packs.py list
            the packs' names, one a line
    python3 -B tools/addon_packs.py digest <pack>
            the pack's content digest: the sha256 over every file's path and sha256 as `stage` places it, its TOC as
            committed among them (so a new ## Interface, which a game patch brings and without which the game does not
            load the pack, is a change, and the release's version and the stores' project ids are none: the same
            content gives the same digest at every release, whatever the variables say)
    python3 -B tools/addon_packs.py stage <pack> --version vN --out DIR
            the pack's files in DIR (new or empty) with its TOC at the top, a .pkgmeta (package-as the pack, its
            changelog) and CHANGELOG.md, committed and tagged <version>, which the packager writes into the TOC's
            ## Version and uses as the upload's label. Prints the digest.

A pack's version is its own, a number that goes up by one each time its content changes (v1, v2, ...: addon-release.yml
keeps the number with the digest on helper-updater, packs job): it follows no tag of the add-on's, so a pack can go to a store before a
release of the add-on needs it, and a pack a store lacks is staged again at that version and gives the same files the
others hold. Needs git."""
import argparse
import hashlib
import os
import re
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
# folder: the pack's folder in AddOns and its zip, with <folder>.toc in the repository's folder of that name;
# sources: (repository folder, folder inside the pack), every file under each but `skip` and dot-files;
# title and words: the upload's changelog, a line a player reads on the store's file page
PACKS = {
    "QuestPump_Voice": {
        "sources": (("QuestPump_Voice", ""), ("Media/Voice", "Takes")),
        "skip": (),
        "title": "QuestPump Voice",
        "words": "QuestPump's recorded voice, {version}.",
    },
    "QuestPump_FigArmor": {
        "sources": (("QuestPump_FigArmor", ""),),
        # the figure build's own table, which the game never reads (tools/figures/build.py)
        "skip": ("pack.json",),
        "title": "QuestPump Figures",
        "words": "QuestPump's animated move figures, {version}.",
    },
}
VERSION = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")


def fail(why):
    sys.exit(f"addon_packs: {why}")


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def toc(pack):
    """The pack's TOC as committed: the stores' project ids are the release's, never the file's."""
    with open(os.path.join(ROOT, pack, f"{pack}.toc"), "rb") as fh:
        data = fh.read()
    for name in ("X-Curse-Project-ID", "X-Wago-ID"):
        if f"## {name}:".encode() in data:
            fail(f"{pack}/{pack}.toc names ## {name}: the release passes the project to the packager "
                 "(addon-release.yml), so the pack's content is the same whatever project takes it")
    return data


def files(pack):
    """Every file of the pack but its TOC: its path inside the pack and its path in the repository, by the first."""
    out = {}
    for source, dest in PACKS[pack]["sources"]:
        top = os.path.join(ROOT, source)
        if not os.path.isdir(top):
            fail(f"{source}/ is missing")
        for folder, dirs, names in os.walk(top):
            dirs[:] = sorted(d for d in dirs if not d.startswith("."))
            for name in sorted(names):
                rel = os.path.relpath(os.path.join(folder, name), top).replace(os.sep, "/")
                if name.startswith(".") or rel in PACKS[pack]["skip"] or (not dest and rel == f"{pack}.toc"):
                    continue
                at = f"{dest}/{rel}" if dest else rel
                if at in out:
                    fail(f"{pack}: {at} comes from two folders")
                out[at] = os.path.join(folder, name)
    return dict(sorted(out.items()))


def listed(pack, toc_bytes, held):
    """Every file the TOC loads is in the pack."""
    for line in toc_bytes.decode("utf-8").splitlines():
        line = line.strip().replace("\\", "/")
        if line and not line.startswith("#") and line not in held:
            fail(f"{pack}/{pack}.toc loads {line}, which the pack lacks")


def digest(pack, toc_bytes, held):
    total = hashlib.sha256()
    total.update(f"{sha256(toc_bytes)}  {pack}.toc\n".encode())
    for at, path in held.items():
        h = hashlib.sha256()
        with open(path, "rb") as fh:
            for chunk in iter(lambda: fh.read(1 << 20), b""):
                h.update(chunk)
        total.update(f"{h.hexdigest()}  {at}\n".encode())
    return total.hexdigest()


def git(out, *args):
    # a repository of its own: no hook, signing or line-end setting of the Mac's or the runner's reaches it
    subprocess.run(["git", "-C", out, "-c", "core.hooksPath=/dev/null", "-c", "core.autocrlf=false",
                    "-c", "core.compression=0", "-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false",
                    "-c", "user.name=Pluxia GmbH", "-c", "user.email=contact@questpump.com", *args], check=True)


def stage(pack, version, out, toc_bytes, held):
    if not VERSION.match(version):
        fail(f"{version!r} is no version a tag can be")
    if os.path.isdir(out) and os.listdir(out):
        fail(f"{out} is not empty")
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, f"{pack}.toc"), "wb") as fh:
        fh.write(toc_bytes)
    for at, path in held.items():
        to = os.path.join(out, at)
        os.makedirs(os.path.dirname(to), exist_ok=True)
        shutil.copyfile(path, to)
    with open(os.path.join(out, ".pkgmeta"), "w", encoding="utf-8") as fh:
        fh.write(f"package-as: {pack}\nenable-nolib-creation: no\nmanual-changelog:\n  filename: CHANGELOG.md\n"
                 "  markup-type: markdown\nignore:\n  - CHANGELOG.md\n")
    with open(os.path.join(out, "CHANGELOG.md"), "w", encoding="utf-8") as fh:
        fh.write(f"# {PACKS[pack]['title']}\n\n{PACKS[pack]['words'].format(version=version)}\n")
    git(out, "init", "-q")
    git(out, "add", "-A")
    git(out, "commit", "-q", "-m", f"{pack} {version}")
    git(out, "tag", version)


def main(argv):
    parser = argparse.ArgumentParser(prog="tools/addon_packs.py", description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("list")
    for name in ("digest", "stage"):
        command = commands.add_parser(name)
        command.add_argument("pack", choices=sorted(PACKS))
        if name == "stage":
            command.add_argument("--version", required=True)
            command.add_argument("--out", required=True)
    args = parser.parse_args(argv)
    if args.command == "list":
        print("\n".join(PACKS))
        return
    toc_bytes = toc(args.pack)
    held = files(args.pack)
    listed(args.pack, toc_bytes, held)
    if args.command == "stage":
        stage(args.pack, args.version, args.out, toc_bytes, held)
    print(digest(args.pack, toc_bytes, held))


if __name__ == "__main__":
    main(sys.argv[1:])
