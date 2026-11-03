"""The add-on's update feed (addon-release.yml's feed job, in the public repository): addon.json for one release's zip,
the packager's zip byte for byte, which the permanent helper-updater release carries beside it as
QuestPump-addon.zip, with addon.json.sig, its signature by the updater's key (`tauri signer sign --app-version
<version>`: base64 of a minisign signature whose trusted comment ends in version:<version>). QuestPump on this computer
reads the feed, checks the signature as tauri-plugin-updater checks its downloads, against tauri.conf.json's
plugins.updater.pubkey, and installs or updates the add-on from it.

    python3 -B tools/addon_feed.py --zip <packager zip> --version X.Y.Z --pub-date <RFC 3339> --phone-apps true|false
        --repo <owner/name> --out addon.json

addon.json, with the fields in this order:

    version     "X.Y.Z", the release's tag without its "v" (the zip's QuestPump.toc ## Version, the tag itself, must
                equal it once its "v" goes)
    pub_date    the addon release's publication, RFC 3339 in UTC ("2026-11-04T10:00:00Z"); a republished feed keeps it
    interfaces  QuestPump.toc's ## Interface list as numbers ([16001, 120105])
    phone_apps  whether the phone apps can be installed from the stores (the repository variable PHONE_APPS)
    zip         {"url": the helper-updater release's QuestPump-addon.zip, "size": bytes, "sha256": hex}
    files       [{"path": "QuestPump/QuestPump.toc", "size": bytes, "sha256": hex}, ...]: every file in the zip, by path

Fails when the version is no X.Y.Z, the date no RFC 3339 time, the zip holds a path outside the three top-level folders
(.pkgmeta: QuestPump and its move-folders QuestPump_Lines and QuestPump_FigArmor), an absolute path, a ".." part, a
backslash or a path twice, lacks a folder's TOC, or its QuestPump.toc names another version or no interface."""
import argparse
import datetime
import hashlib
import json
import re
import sys
import zipfile

VERSION = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+$")
# The zip's top-level folders (.pkgmeta: package-as and move-folders; tools/package-check.sh checks the same three)
FOLDERS = ("QuestPump", "QuestPump_Lines", "QuestPump_FigArmor")
ASSET = "QuestPump-addon.zip"


def digest(stream):
    h, size = hashlib.sha256(), 0
    for chunk in iter(lambda: stream.read(1 << 20), b""):
        h.update(chunk)
        size += len(chunk)
    return size, h.hexdigest()


def toc_field(text, name):
    for line in text.splitlines():
        if line.startswith(f"## {name}:"):
            return line.split(":", 1)[1].strip()
    return None


def feed(zip_path, version, pub_date, phone_apps, repo):
    if not VERSION.match(version):
        sys.exit(f"addon_feed: {version!r} is no X.Y.Z version")
    try:
        when = datetime.datetime.fromisoformat(pub_date.replace("Z", "+00:00"))
    except ValueError:
        sys.exit(f"addon_feed: {pub_date!r} is no RFC 3339 time")
    if when.tzinfo is None:
        sys.exit(f"addon_feed: {pub_date!r} names no time zone")
    files, seen = [], set()
    with zipfile.ZipFile(zip_path) as z:
        for info in sorted(z.infolist(), key=lambda i: i.filename):
            name = info.filename
            if name.endswith("/"):
                continue
            parts = name.split("/")
            if name.startswith("/") or "\\" in name or ".." in parts or parts[0] not in FOLDERS or len(parts) < 2:
                sys.exit(f"addon_feed: {name!r} is no path inside {', '.join(FOLDERS)}")
            if name in seen:
                sys.exit(f"addon_feed: {name!r} is in the zip twice")
            seen.add(name)
            with z.open(info) as fh:
                size, sha = digest(fh)
            files.append({"path": name, "size": size, "sha256": sha})
        for folder in FOLDERS:
            if f"{folder}/{folder}.toc" not in seen:
                sys.exit(f"addon_feed: the zip has no {folder}/{folder}.toc")
        toc = z.read("QuestPump/QuestPump.toc").decode("utf-8-sig")
    # the packager writes the tag as it is ("v1.0.0"); versions compare without their "v"
    toc_version = toc_field(toc, "Version") or ""
    if toc_version.removeprefix("v") != version:
        sys.exit(f"addon_feed: QuestPump.toc's ## Version is {toc_version!r}, not {version!r}")
    interfaces = [int(i) for i in re.findall(r"[0-9]+", toc_field(toc, "Interface") or "")]
    if not interfaces:
        sys.exit("addon_feed: QuestPump.toc has no ## Interface")
    with open(zip_path, "rb") as fh:
        size, sha = digest(fh)
    return {
        "version": version,
        "pub_date": when.astimezone(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "interfaces": interfaces,
        "phone_apps": phone_apps,
        "zip": {"url": f"https://github.com/{repo}/releases/download/helper-updater/{ASSET}", "size": size,
                "sha256": sha},
        "files": files,
    }


def main(argv):
    parser = argparse.ArgumentParser(prog="tools/addon_feed.py", description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--zip", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--pub-date", required=True)
    parser.add_argument("--phone-apps", required=True, choices=("true", "false"))
    parser.add_argument("--repo", required=True)
    parser.add_argument("--out", required=True)
    args = parser.parse_args(argv)
    doc = feed(args.zip, args.version, args.pub_date, args.phone_apps == "true", args.repo)
    with open(args.out, "w", encoding="utf-8") as fh:
        fh.write(json.dumps(doc, indent=1) + "\n")
    print(f"{args.out}: {doc['version']}, {len(doc['files'])} files, zip {doc['zip']['size']} bytes, "
          f"phone_apps {str(doc['phone_apps']).lower()}")


if __name__ == "__main__":
    main(sys.argv[1:])
