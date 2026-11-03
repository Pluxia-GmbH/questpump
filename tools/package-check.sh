#!/bin/sh
# The packager's zip (docs/plan.md slice 2, "The pack's checks"): all three top-level folders (the addon, the quests'
# lines, the figure pack) with every file each folder's TOC lists (a file git does not track, or one .pkgmeta ignores,
# loads as missing in the client) and every file an XML of those loads (a library's embed.xml, slice 15), pack.json left
# out, and under the 300 MB budget (docs/plan.md slice 2, "Shipping": CurseForge takes files up to 2 GB and Wago states
# no limit, so the budget is the download a player takes on each update; 40 MB until slice 21, which its poses outgrew,
# 80 MB from 2026-10-04 until the move guides' voice, about 30 MB, came on 2026-10-09, and 100 MB until the guides were
# finished as every other line is, 66.9 MB, on 2026-10-10, 150 MB until the ladder gaps' 34 moves on 2026-10-12, and
# 200 MB until slice 47's 140 poses, their figures and voice, on 2026-10-20). Run after a dry run (make package, and CI
# before a tag's upload).
set -eu
zip=$(ls -t .release/*.zip | head -1)
bytes=$(wc -c < "$zip")
list=$(unzip -Z1 "$zip")

# Each <Script file> and <Include file> of an XML in the zip, relative to that XML's folder, and the XMLs they include.
check_xml() {
	xml_dir=$(dirname "$1")
	for g in $(unzip -p "$zip" "$1" | tr -d '\r' | grep -oE "<(Script|Include)[^>]*file=['\"][^'\"]+['\"]" |
		sed -E "s/.*file=['\"]([^'\"]+)['\"].*/\1/" | tr '\\' '/'); do
		echo "$list" | grep -qxF "$xml_dir/$g" || { echo "$xml_dir/$g, loaded by $1, is not in $zip" >&2; exit 1; }
		case $g in *.xml) (check_xml "$xml_dir/$g") || exit 1 ;; esac
	done
}
for dir in QuestPump QuestPump_Lines QuestPump_FigArmor; do
	toc=$dir/$dir.toc
	[ "$dir" = QuestPump ] && toc=QuestPump.toc
	echo "$list" | grep -qxF "$dir/$dir.toc" || { echo "no $dir/ in $zip" >&2; exit 1; }
	for f in $(grep -vE '^[[:space:]]*(#|$)' "$toc" | tr '\\' '/' | tr -d '\r'); do
		echo "$list" | grep -qxF "$dir/$f" || { echo "$dir/$f, listed in $toc, is not in $zip" >&2; exit 1; }
		case $f in *.xml) check_xml "$dir/$f" ;; esac
	done
done
if echo "$list" | grep -q 'pack.json'; then echo "pack.json is in the zip" >&2; exit 1; fi
[ "$bytes" -le 314572800 ] || { echo "$zip is $bytes bytes, over the 300 MB budget" >&2; exit 1; }
echo "package check: $zip, $bytes bytes"
