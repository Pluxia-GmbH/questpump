# shellcheck shell=bash
# The add-on release's state on the permanent helper-updater release (addon-release.yml; docs/plan.md slice 71): small
# JSON files that say what each store holds, so a run sends a store only what it lacks and a run made again sends
# nothing twice: pack-<name>.json (a pack's version, its content's digest, the digest CurseForge and Wago hold and the
# one Wago refused) and addon-stores.json (the add-on's tag each store took).
#
# Sourced by a step's bash (`. tools/release-state.sh`), with GH_TOKEN and GH_REPO set:
#     state_get <asset> <file>   the asset into <file>; {} only where the release holds neither it nor its .next copy,
#                                which is "not there yet". Any other failure (the list or the download does not come,
#                                the file is no JSON object) stops the step: a state read as empty would give an
#                                unchanged pack a new version and send it again.
#     state_put <asset> <file>   <file> onto the release as <asset>. GitHub replaces an asset by deleting it first, so
#                                the copy <asset>.next goes up before and comes down after: a write cut between the
#                                two leaves the .next copy, which state_get reads.

state_get() {
	local names from=""
	if ! names="$(gh release view helper-updater --json assets --jq '.assets[].name')"; then
		echo "::error::helper-updater's files could not be listed, so $1 was not read: nothing is sent on a guess" >&2
		return 1
	fi
	if grep -qxF "$1" <<< "$names"; then
		from="$1"
	elif grep -qxF "$1.next" <<< "$names"; then
		from="$1.next"
	fi
	if [ -z "$from" ]; then
		echo '{}' > "$2"
		return 0
	fi
	if ! gh release download helper-updater --pattern "$from" --output "$2" --clobber ||
		! jq -e 'type == "object"' "$2" > /dev/null; then
		echo "::error::$from on helper-updater did not download as a JSON object: nothing is sent on a guess" >&2
		return 1
	fi
}

state_put() {
	local dir
	dir="$(mktemp -d)"
	cp "$2" "$dir/$1.next"
	cp "$2" "$dir/$1"
	if ! gh release upload helper-updater "$dir/$1.next" --clobber ||
		! gh release upload helper-updater "$dir/$1" --clobber; then
		echo "::error::$1 was not written to helper-updater" >&2
		return 1
	fi
	gh release delete-asset helper-updater "$1.next" --yes || true
}
