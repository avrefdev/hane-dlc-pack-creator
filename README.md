# Hane DLC Pack Creator

Creates ready to fill GTA V add-on DLC packs (`dlc.rpf`) for map and vehicle mods.

A rewrite of skylumz's Dlc Pack Creator 1.1 with the same output for map packs, fixed vehicle packs and a remembered output folder.

## Usage

1. On first launch, select `GTA5.exe` (or `GTA5_Enhanced.exe`). The location is checked on every launch and asked again if the game moved. It can be changed in **Settings**.
2. Type a DLC name (no spaces), pick **Map** or **Vehicle**.
3. Click **Create pack**. The pack is written to `<GTA V>\mods\update\x64\dlcpacks\<name>\dlc.rpf`.
4. Copy the `dlclist.xml` line from the result and follow the steps in the **Guide** tab.

### Linking a FiveM map resource

For map packs you can select a FiveM resource folder. Its GTA files are packed automatically:

- `ymt`, `ytd`, `ytyp`, `ymap`, `ynv`, `ymf` go into `<name>_metadata.rpf`
- other GTA files (`ydr`, `ydd`, `ybn`, `yft`...) go into `<name>.rpf`
- `.lua` and other FiveM files are skipped, and files with the same name stop the build

The link is saved. When the resource changes, press **Sync** on the **Linked** page to rebuild the pack. The previous pack is kept as `dlc.rpf.bak`.

## What is inside a pack

| Type | Contents |
| --- | --- |
| Map | `content.xml`, `setup2.xml`, `x64\levels\gta5\<name>.rpf`, `x64\levels\gta5\<name>_metadata.rpf` |
| Vehicle | `content.xml`, `setup2.xml`, `data\`, `x64\vehicles.rpf`, `x64\data\lang\<language>dlc.rpf` (13 languages, each with an empty `global.gxt2`) |

Archives are written as RPF7 with OPEN encryption, the same format OpenIV and CodeWalker use for mods.

## Changes from the original

- The GTA V folder is selected once and packs go straight into `mods\update\x64\dlcpacks`.
- Map packs: identical `content.xml` and `setup2.xml`. The timestamp uses `MM/dd/yyyy HH:mm:ss`.
- Vehicle packs:
  - `content.xml` used unclosed `txdToLoad`, `txdToUnLoad`, `residentResources` and `unregisterResources` tags. They are now self closing.
  - `setup2.xml` never closed `contentChangeSetGroups`. It is closed now.
  - The change set was named `<name>AUTOGEN` in `content.xml` but `<name>_AUTOGEN` in `setup2.xml`, so the files were never enabled. Both use `<name>_AUTOGEN` now.
  - Language archives moved from `x64\data` to `x64\data\lang`, `russion` is now `russian`, and `german` and `chinesesimp` were added.
  - `global.gxt2` is a valid empty GXT2 file instead of the text `TODO`.
- Names with non English letters or characters Windows does not allow are rejected with a clear message.

## Building

Requirements: Node.js 20+, Rust (stable), Visual Studio Build Tools, WebView2.

```bash
npm install
npm run assets
npm run build
```

The exe is copied to `release\` and to `..\exe\`.

## License

MIT. See `LICENSE` and `THIRD_PARTY.md`.
