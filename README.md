# LiquidLauncher

A local launcher for this build of LiquidBounce on Minecraft 1.21.11 / Fabric.

This is a stripped-down fork of [CCBlueX/LiquidLauncher](https://github.com/CCBlueX/LiquidLauncher).
It starts the LiquidBounce jar that ships with it and downloads only what Minecraft needs.
There is no account service, marketplace, embedded download view, telemetry or self-updater.

## What it does

- **Offline accounts only.** Pick a username (1–16 letters, digits, underscores); the account is
  stored locally and the UUID is derived the same way Minecraft derives it for offline play.
- **One supported version:** Minecraft 1.21.11 with the newest stable Fabric loader.
- **LiquidBounce comes from the AppImage.** `liquidbounce-0.37.0-ryoku.jar` is bundled as a
  resource, copied into the data folder and started from there. A different jar can be selected in
  _Settings → General_.
- **Instances.** Each instance has its own worlds, resource packs, mods and settings in
  `<data>/gameDir/<id>`. Minecraft, Fabric and Java are shared between instances.
- **Mods from Modrinth.** Search and install Fabric mods for 1.21.11, add local jars, turn mods on
  and off, update them, and see which of your files Modrinth knows. Mods live in
  `<data>/custom_mods/<instance>-1.21.11` and are copied into the instance's `mods` folder at launch.
- **Cracked skin switcher with 3D preview.** Manage local skin library in _Settings → Skins_
  like in Modrinth App. Real-time 3D interactive preview with orbit rotation, layer toggling, and
  Classic (4px) vs. Slim (3px) model support. Import from local PNG files or fetch skins by Minecraft
  username directly. Active skins automatically sync to LiquidBounce and CustomSkinLoader for offline play.
- **Appearance.** Choose a background picture in _Settings → Appearance_ and set its tint colour,
  opacity and blur. The picture is copied into the launcher's config directory, so the original
  file can be moved or deleted.
- **Log viewer.** A local log viewer with auto-scroll; logs stay on the machine.

## Where data goes

| What | Where |
| --- | --- |
| Options, background picture, skin library, version cache | the launcher config dir (`~/.config/liquidlauncher` on Linux) |
| Minecraft, libraries, assets, runtimes, mod cache, active skins | the data dir (`~/.local/share/liquidlauncher` on Linux) |
| Instance list | `<data>/instances.json` |
| Instance game directories | `<data>/gameDir/<id>` |
| Jars added per instance | `<data>/custom_mods/<id>-1.21.11` |

The data directory can be changed in _Settings → General_.

## Network use

The launcher talks to these hosts, and only these:

- `launchermeta.mojang.com`, `piston-meta.mojang.com`, `libraries.minecraft.net`,
  `resources.download.minecraft.net` — Minecraft version metadata, libraries and assets
- `meta.fabricmc.net` — Fabric loader version and profile
- `api.modrinth.com` — Fabric API and mods you search for or install
- `api.adoptium.net`, `api.azul.com`, `download.oracle.com` — the Java runtime, only when one has
  to be downloaded

Nothing is uploaded. There is no CCBlueX build, auth, paste, marketplace or update endpoint left in
the code.

## Build

The launcher is Tauri 2 + Rust with a Svelte frontend. Building an AppImage needs Rust, bun, and the
usual WebKitGTK development packages.

```sh
bun install
bun run build                      # frontend only
bun run tauri build --bundles appimage
```

The bundled client jar is read from `../liquidbounce-0.37.0-ryoku.jar` (see
`src-tauri/tauri.conf.json` → `bundle.resources`). The AppImage is written to
`src-tauri/target/release/bundle/appimage/`.

## License

This project is subject to the [GNU General Public License v3.0](LICENSE). This does only apply for source code located directly in this clean repository. During the development and compilation process, additional source code may be used to which we have obtained no rights. Such code is not covered by the GPL license.

For those who are unfamiliar with the license, here is a summary of its main points. This is by no means legal advice nor legally binding.

You are allowed to

- use
- share
- modify

this project entirely or partially for free and even commercially. However, please consider the following:

- **You must disclose the source code of your modified work and the source code you took from this project. This means you are not allowed to use code from this project (even partially) in a closed-source (or even obfuscated) application.**
- **Your modified application must also be licensed under the GPL**

Do the above and share your source code with everyone; just like we do.

## Icons

We use [Clarity Line Icons](https://www.svgrepo.com/collection/clarity-line-icons/) for this project.

## Commits

`type(scope): subject`, lowercase, no trailing period.

`feat` `fix` `refactor` `chore` `docs`

Add a short body when the subject alone leaves the next reader guessing.
