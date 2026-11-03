# QuestPump's cask (the desktop app, built from helper/) in the tap Pluxia-GmbH/homebrew-tap. The helper's release
# workflow (.github/workflows/helper-release.yml in Pluxia-GmbH/questpump) writes Casks/questpump.rb there from the
# template helper/packaging/homebrew/questpump.rb, with each release's version and the two disk images' checksums:
# change the template, never the tap's copy.
cask "questpump" do
  arch arm: "aarch64", intel: "x64"

  version "@VERSION@"
  sha256 arm:   "@SHA256_ARM@",
         intel: "@SHA256_INTEL@"

  url "https://github.com/Pluxia-GmbH/questpump/releases/download/helper-v#{version}/QuestPump_#{version}_#{arch}.dmg",
      verified: "github.com/Pluxia-GmbH/questpump/"
  name "QuestPump"
  desc "Exercise pumps between games, synced with the add-on and the phone app"
  homepage "https://questpump.com/helper"

  livecheck do
    url :url
    regex(/^helper[._-]v?(\d+(?:\.\d+)+)$/i)
    strategy :github_releases
  end

  # the app installs its own signed updates
  auto_updates true
  depends_on macos: :sonoma

  app "QuestPump.app"

  # start at login is a login item registered through SMAppService, so there is no LaunchAgent file to remove
  zap trash: [
    "~/Library/Application Support/QuestPump",
    "~/Library/Caches/ai.pluxia.questpump.bridge",
    "~/Library/Logs/ai.pluxia.questpump.bridge",
    "~/Library/WebKit/ai.pluxia.questpump.bridge",
  ]
end
