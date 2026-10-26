# Hane DLC Pack Creator

Creates ready to fill GTA V add-on DLC packs (`dlc.rpf`) for map and vehicle mods.

A rewrite of skylumz's Dlc Pack Creator 1.1 with the same output for map packs, fixed vehicle packs and a remembered output folder.

## Usage

1. Type a DLC name (no spaces), pick **Map** or **Vehicle**.
2. Pick an output folder from the **Save to** list or add a new one. The list keeps up to 8 folders and the last one you used is selected on the next launch.
3. Click **Create pack**. The pack is written to `<folder>\<name>\dlc.rpf`.
4. Copy the `dlclist.xml` line from the result and follow the steps in the **Guide** tab.

## What is inside a pack

| Type | Contents |
| --- | --- |
| Map | `content.xml`, `setup2.xml`, `x64\levels\gta5\<name>.rpf`, `x64\levels\gta5\<name>_metadata.rpf` |
| Vehicle | `content.xml`, `setup2.xml`, `data\`, `x64\vehicles.rpf`, `x64\data\lang\<language>dlc.rpf` (13 languages, each with an empty `global.gxt2`) |

Archives are written as RPF7 with OPEN encryption, the same format OpenIV and CodeWalker use for mods.

## Changes from the original

- Output folders are saved in a list and the last used one is preselected on every launch.
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
