#!/bin/sh
# The packager's zips (docs/plan.md slice 2, "The pack's checks"; three zips since slice 71): the add-on's own (QuestPump
# and the quests' lines) and each media pack's (tools/addon_packs.py: the voice, the move figures), each holding its
# folders alone with every file each folder's TOC lists (a file git does not track, or one .pkgmeta ignores, loads as
# missing in the client) and every file an XML of those loads (a library's embed.xml, slice 15), the add-on's without a
# voice take or a figure sheet, a pack's with its media, pack.json left out, and all of them together under the 300 MB
# budget (docs/plan.md slice 2, "Shipping": the budget is the download a first install takes; CurseForge takes files up
# to 2 GB, and Wago, which states no limit, refused 1.0.0's 238 MB zip at its Cloudflare front (413) and took another
# add-on's 101,799,121 bytes, under the 100 MiB that front allows on its smaller plans, so a zip over 100 MiB gets a
# note: whether Wago takes it is not known until an upload; 40 MB until slice 21, which its poses outgrew, 80 MB from
# 2026-10-04 until the move guides' voice, about 30 MB, came on 2026-10-09, and 100 MB until the guides were finished as
# every other line is, 66.9 MB, on 2026-10-10, 150 MB until the ladder gaps' 34 moves on 2026-10-12, and 200 MB until
# slice 47's 140 poses, their figures and voice, on 2026-10-20). Run after a dry run (make package, and CI before a
# tag's upload):
#     sh tools/package-check.sh            the add-on's zip and every pack's
#     sh tools/package-check.sh <name>...  those alone: QuestPump, or a pack's folder
set -eu
# shellcheck disable=SC2046 # one word a pack
[ $# -gt 0 ] || set -- QuestPump $(python3 -B tools/addon_packs.py list)
total=0

# Each <Script file> and <Include file> of an XML in the zip, relative to that XML's folder, and the XMLs they include.
check_xml() {
	xml_dir=$(dirname "$1")
	for g in $(unzip -p "$zip" "$1" | tr -d '\r' | grep -oE "<(Script|Include)[^>]*file=['\"][^'\"]+['\"]" |
		sed -E "s/.*file=['\"]([^'\"]+)['\"].*/\1/" | tr '\\' '/'); do
		echo "$list" | grep -qxF "$xml_dir/$g" || { echo "$xml_dir/$g, loaded by $1, is not in $zip" >&2; exit 1; }
		case $g in *.xml) (check_xml "$xml_dir/$g") || exit 1 ;; esac
	done
}
for name in "$@"; do
	zip=$(ls -t .release/"$name"-*.zip 2>/dev/null | head -1)
	[ -n "$zip" ] || { echo "no .release/$name-*.zip" >&2; exit 1; }
	bytes=$(wc -c < "$zip" | tr -d ' ')
	list=$(unzip -Z1 "$zip")
	# the folders the zip holds, and for a pack the pattern of its media's paths
	case $name in
	QuestPump) dirs="QuestPump QuestPump_Lines"; media="" ;;
	QuestPump_Voice) dirs=$name; media='^QuestPump_Voice/Takes/.*\.ogg$' ;;
	QuestPump_FigArmor) dirs=$name; media='^QuestPump_FigArmor/s/.*\.png$' ;;
	*) echo "$name is neither QuestPump nor a pack" >&2; exit 1 ;;
	esac
	for top in $(echo "$list" | cut -d/ -f1 | sort -u); do
		case " $dirs " in *" $top "*) ;; *) echo "$zip holds $top/, not only $dirs" >&2; exit 1 ;; esac
	done
	for dir in $dirs; do
		toc=$dir/$dir.toc
		[ "$dir" = QuestPump ] && toc=QuestPump.toc
		echo "$list" | grep -qxF "$dir/$dir.toc" || { echo "no $dir/ in $zip" >&2; exit 1; }
		for f in $(grep -vE '^[[:space:]]*(#|$)' "$toc" | tr '\\' '/' | tr -d '\r'); do
			echo "$list" | grep -qxF "$dir/$f" || { echo "$dir/$f, listed in $toc, is not in $zip" >&2; exit 1; }
			case $f in *.xml) check_xml "$dir/$f" ;; esac
		done
	done
	if [ -n "$media" ]; then
		echo "$list" | grep -qE "$media" || { echo "$zip holds none of its media ($media)" >&2; exit 1; }
	elif echo "$list" | grep -qE '\.ogg$'; then
		echo "$zip holds voice takes: they ship in QuestPump_Voice" >&2; exit 1
	fi
	if echo "$list" | grep -q 'pack.json'; then echo "pack.json is in $zip" >&2; exit 1; fi
	[ "$bytes" -le 104857600 ] || echo "note: $zip is over 100 MiB: Wago may refuse it (it is known to take 97 MiB)"
	echo "package check: $zip, $bytes bytes"
	total=$((total + bytes))
done
[ "$total" -le 314572800 ] || { echo "the zips are $total bytes together, over the 300 MB budget" >&2; exit 1; }
echo "package check: $# zips, $total bytes together"
