"""The add-on's update feed (addon-release.yml's feed job, in the public repository): addon-v2.json for one release,
its three zips (the add-on's own and its two packs', the packager's, tools/addon_packs.py) joined into one,
QuestPump-addon-v2.zip, which the permanent helper-updater release carries beside it with addon-v2.json.sig, its
signature by the updater's key (`tauri signer sign --app-version <version>`: base64 of a minisign signature whose
trusted comment ends in version:<version>). QuestPump on this computer reads the feed, checks the signature as
tauri-plugin-updater checks its downloads, against tauri.conf.json's plugins.updater.pubkey, and installs or updates the
add-on and its packs from it file by file, so it fetches only what changed and one zip serves every folder. The feed's
files are the stores' files, each with the stores' bytes; the joined zip is its own (entries by path, .ogg and .png
stored, one fixed time, so the same files give the same zip).

The first feed, addon.json with QuestPump-addon.zip (one zip of three folders, until slice 71), stays on helper-updater
as its last release left it: a desktop app from before the packs reads it and refuses a fourth folder.

    python3 -B tools/addon_feed.py --zip <the add-on's zip> [--pack <a pack's zip> ...] --version X.Y.Z
        --pub-date <RFC 3339> --phone-apps true|false --repo <owner/name> --merged QuestPump-addon-v2.zip
        --out addon-v2.json

Where the add-on's own zip is from before the packs (it holds the figure pack's folder, or voice takes under
QuestPump/Media/Voice), the packs' zips are left out, with a line that says so: that release carries its figures and
its voice itself, whatever the packs' state has moved to since.

addon-v2.json, with the fields in this order:

    version     "X.Y.Z", the release's tag without its "v" (the add-on's QuestPump.toc ## Version, the tag itself, must
                equal it once its "v" goes; a pack's TOC names the pack's own version, a number)
    pub_date    the addon release's publication, RFC 3339 in UTC ("2026-11-04T10:00:00Z"); a republished feed keeps it
    interfaces  QuestPump.toc's ## Interface list as numbers ([16001, 120105])
    phone_apps  whether the phone apps can be installed from the stores (the repository variable PHONE_APPS)
    zip         {"url": the helper-updater release's QuestPump-addon-v2.zip, "size": bytes, "sha256": hex}
    files       [{"path": "QuestPump/QuestPump.toc", "size": bytes, "sha256": hex}, ...]: every file, by path

Fails when the version is no X.Y.Z, the date no RFC 3339 time, a zip holds a path the desktop app would refuse
(questpump-core's addon.rs `safe_path`: outside the four top-level folders, FOLDERS, an absolute path, an empty part, a
"." or ".." part, a backslash, a colon or a control character) or one Windows cannot hold (a reserved name such as
CON or LPT1, a part that ends in a dot or a space) or tell from another (two paths that differ by case alone), a path
comes twice, QuestPump, the quests' lines or the figure pack lacks its TOC, no voice take is there (the voice pack's,
or QuestPump/Media/Voice's in a release from before the packs), or QuestPump.toc names another version or no interface:
a release the app would refuse fails here, at build time, and a feed never takes an install's figures or voice away."""
import argparse
import datetime
import hashlib
import json
import re
import sys
import unicodedata
import zipfile

VERSION = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+$")
# The folders a release puts into Interface/AddOns: the add-on's zip holds the first two (.pkgmeta: package-as and
# move-folders), each pack's its own (tools/addon_packs.py PACKS; the desktop app's addon.rs FOLDERS lists the same)
FOLDERS = ("QuestPump", "QuestPump_Lines", "QuestPump_Voice", "QuestPump_FigArmor")
# The folders every release ships with their TOC; the voice came inside QuestPump until the packs
TOCS = ("QuestPump", "QuestPump_Lines", "QuestPump_FigArmor")
VOICE = ("QuestPump_Voice/Takes/", "QuestPump/Media/Voice/")
ASSET = "QuestPump-addon-v2.zip"
# already compressed: stored, as the packager's dry run stores them (ZIPOPT)
STORED = (".ogg", ".png")
# The names Windows keeps for devices, whatever follows a dot
RESERVED = re.compile(r"^(con|prn|aux|nul|com[1-9]|lpt[1-9])(\..*)?$", re.IGNORECASE)


def toc_field(text, name):
    for line in text.splitlines():
        if line.startswith(f"## {name}:"):
            return line.split(":", 1)[1].strip()
    return None


def unsafe(name):
    """Why the desktop app or Windows would refuse this path, or None."""
    parts = name.split("/")
    if len(parts) < 2 or parts[0] not in FOLDERS:
        return f"is no file inside {', '.join(FOLDERS)}"
    for part in parts:
        if part in ("", ".", ".."):
            return "has an empty, '.' or '..' part"
        if any(c in "\\:" or unicodedata.category(c) == "Cc" for c in part):
            return "has a backslash, a colon or a control character"
        if RESERVED.match(part) or part.endswith((".", " ")):
            return f"has a part Windows cannot hold ({part!r})"
    return None


def read(path, held, folded):
    """Every file of the zip at `path` into `held`, each path one the app and Windows take."""
    with zipfile.ZipFile(path) as z:
        for info in z.infolist():
            name = info.filename
            if name.endswith("/"):
                continue
            why = unsafe(name)
            if why:
                sys.exit(f"addon_feed: {name!r} in {path} {why}")
            if name in held:
                sys.exit(f"addon_feed: {name!r} comes twice")
            twin = folded.setdefault(name.casefold(), name)
            if twin != name:
                sys.exit(f"addon_feed: {name!r} and {twin!r} differ by case alone")
            held[name] = z.read(info)


def merge(addon, packs, merged):
    """Every file of the add-on's zip and of the packs' into the zip `merged`, by path; each one's path, size and
    sha256, and QuestPump.toc's text."""
    held, folded = {}, {}
    read(addon, held, folded)
    # a release from before the packs: its one zip holds more than QuestPump and the quests' lines
    if any(name.split("/")[0] not in FOLDERS[:2] or name.startswith(VOICE[1]) for name in held):
        for path in packs:
            print(f"{path}: left out, the add-on's own zip is from before the packs and carries its media")
    else:
        for path in packs:
            read(path, held, folded)
    for folder in TOCS:
        if f"{folder}/{folder}.toc" not in held:
            sys.exit(f"addon_feed: the zips have no {folder}/{folder}.toc")
    if not any(name.startswith(VOICE) and name.endswith(".ogg") for name in held):
        sys.exit("addon_feed: the zips have no voice take")
    files = []
    with zipfile.ZipFile(merged, "w") as out:
        for name in sorted(held):
            data = held[name]
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED if name.endswith(STORED) else zipfile.ZIP_DEFLATED
            info.create_system = 3
            info.external_attr = 0o644 << 16
            out.writestr(info, data)
            files.append({"path": name, "size": len(data), "sha256": hashlib.sha256(data).hexdigest()})
    return files, held["QuestPump/QuestPump.toc"].decode("utf-8-sig")


def feed(addon, packs, merged, version, pub_date, phone_apps, repo):
    if not VERSION.match(version):
        sys.exit(f"addon_feed: {version!r} is no X.Y.Z version")
    try:
        when = datetime.datetime.fromisoformat(pub_date.replace("Z", "+00:00"))
    except ValueError:
        sys.exit(f"addon_feed: {pub_date!r} is no RFC 3339 time")
    if when.tzinfo is None:
        sys.exit(f"addon_feed: {pub_date!r} names no time zone")
    files, toc = merge(addon, packs, merged)
    # the packager writes the tag as it is ("v1.0.0"); versions compare without their "v"
    toc_version = toc_field(toc, "Version") or ""
    if toc_version.removeprefix("v") != version:
        sys.exit(f"addon_feed: QuestPump.toc's ## Version is {toc_version!r}, not {version!r}")
    interfaces = [int(i) for i in re.findall(r"[0-9]+", toc_field(toc, "Interface") or "")]
    if not interfaces:
        sys.exit("addon_feed: QuestPump.toc has no ## Interface")
    h, size = hashlib.sha256(), 0
    with open(merged, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
            size += len(chunk)
    return {
        "version": version,
        "pub_date": when.astimezone(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "interfaces": interfaces,
        "phone_apps": phone_apps,
        "zip": {"url": f"https://github.com/{repo}/releases/download/helper-updater/{ASSET}", "size": size,
                "sha256": h.hexdigest()},
        "files": files,
    }


def main(argv):
    parser = argparse.ArgumentParser(prog="tools/addon_feed.py", description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--zip", required=True)
    parser.add_argument("--pack", action="append", default=[])
    parser.add_argument("--version", required=True)
    parser.add_argument("--pub-date", required=True)
    parser.add_argument("--phone-apps", required=True, choices=("true", "false"))
    parser.add_argument("--repo", required=True)
    parser.add_argument("--merged", required=True)
    parser.add_argument("--out", required=True)
    args = parser.parse_args(argv)
    doc = feed(args.zip, args.pack, args.merged, args.version, args.pub_date, args.phone_apps == "true", args.repo)
    with open(args.out, "w", encoding="utf-8") as fh:
        fh.write(json.dumps(doc, indent=1) + "\n")
    print(f"{args.out}: {doc['version']}, {len(doc['files'])} files, {args.merged} {doc['zip']['size']} bytes, "
          f"phone_apps {str(doc['phone_apps']).lower()}")


if __name__ == "__main__":
    main(sys.argv[1:])
