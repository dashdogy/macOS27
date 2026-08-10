# Powerflow macOS27

Powerflow macOS27 is an Apple Silicon compatibility and performance distribution of Powerflow. It preserves the existing Powerflow app identity and data locations while adding safe support for the battery diagnostics schema used by macOS 27.

![Powerflow Screenshot](https://raw.githubusercontent.com/lzt1008/powerflow/assets/screenshot.png)

Powerflow monitors the power usage and charging status of your Mac and connected iOS devices.

## Lineage and attribution

This project is intentionally a fork of a fork, with its Git history preserved:

1. [lzt1008/powerflow](https://github.com/lzt1008/powerflow) is the original upstream project, created and maintained by **lzt1008 and the Powerflow contributors**.
2. [lmqferreira/powerflow](https://github.com/lmqferreira/powerflow) is the immediate upstream performance fork, maintained by **Luis Ferreira**.
3. [dashdogy/macOS27](https://github.com/dashdogy/macOS27) adds macOS 27 compatibility and packages the Apple Silicon build as **Powerflow macOS27**.
This macOS27 update was vibe coded using GPT-5.6 Sol.
Powerflow macOS27 does not claim authorship of the upstream work. Copyright remains with the respective authors and contributors. See [NOTICE.md](NOTICE.md) and [LICENSE](LICENSE).

## What this distribution adds

- Safe decoding of both legacy flat IORegistry battery properties and the nested `BatteryData` representation used by macOS 27.
- Typed validation for missing or invalid battery capacities instead of undefined behavior.
- Clean process termination if local battery diagnostics become unreadable.
- Removal of unsafe IORegistry representation transmutes for local Macs and connected iOS devices.
- Native Apple Silicon (`arm64`) release artifacts.

It also retains the immediate upstream fork's performance work, including throttled menu-bar updates, a longer polling interval, delta-based frontend events, no WebKit updates while windows are hidden, CSS-based animations, and lazy chart loading.

## Installation

### Homebrew

Install directly from the macOS27 tap:

```bash
brew install --cask dashdogy/macos27/powerflow-macos27
```

Upgrade later with:

```bash
brew upgrade --cask powerflow-macos27
```

### Manual installation

Download the latest Apple Silicon DMG from [Releases](https://github.com/dashdogy/macOS27/releases), open it, and drag `powerflow.app` into Applications.

The current local-validation build is ad-hoc signed and is not notarized. If macOS blocks the first launch, open **System Settings → Privacy & Security**, select **Open Anyway**, and confirm the prompt. Do not bypass this warning for artifacts downloaded from anywhere other than this repository's release page.

## Compatibility

- Distributed binary: Apple Silicon (`arm64`) only.
- Validated on macOS 27.
- The decoder retains support for the earlier flat IORegistry battery schema.

## Development

The application uses Tauri 2, Rust, Vue 3, TypeScript, and pnpm.

```bash
corepack pnpm install --frozen-lockfile
corepack pnpm tauri dev
```

## License

Powerflow macOS27 and its upstream projects are distributed under the [MIT License](LICENSE). The license and attribution notice must remain with copies or substantial portions of the software.
