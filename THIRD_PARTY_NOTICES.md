# Third-party notices / уведомления

## Horizon

This is a modified build of [Horizon v0.2.92](https://github.com/horizon-rust/horizon/tree/v0.2.92). Copyright (c) 2026 Ludovic Marie and the Horizon contributors; MIT license: [HORIZON-LICENSE.txt](HORIZON-LICENSE.txt). The source archive contains our patch for that exact upstream version, and the upstream source remains available at the link above.

## Rust dependencies and toolchain

[DEPENDENCIES.md](DEPENDENCIES.md) lists exact third-party Rust package versions, declared license expressions and source locations, including the mod's build dependencies. [licenses/CRATE-NOTICES.txt](licenses/CRATE-NOTICES.txt) reproduces the license, copyright and notice files supplied with those packages. Additional standard texts are in `licenses/`. The list may include build-time packages or other packages not present in the final executable.

The Rust standard library and toolchain are distributed under their applicable MIT/Apache-2.0 licenses. Windows binaries were built with the MinGW-w64 GCC toolchain; its copyright and the GCC Runtime Library Exception are included in [licenses/GCC-MinGW-copyright.txt](licenses/GCC-MinGW-copyright.txt) and [licenses/MinGW-w64-copyright.txt](licenses/MinGW-w64-copyright.txt).

### Symphonia (MPL-2.0)

The audio dependency Symphonia 0.5.5 and its `symphonia-core`, `symphonia-codec-vorbis`, `symphonia-format-ogg`, `symphonia-metadata` and `symphonia-utils-xiph` crates are Copyright (c) 2019–2022 The Project Symphonia Developers, licensed under the [Mozilla Public License 2.0](licenses/MPL-2.0.txt). They are unmodified. Their exact corresponding source distributions are provided in `third_party/symphonia-sources/` in the repository and the source archive; they are also available from the versioned crates.io links in DEPENDENCIES.md. The executable form combines these covered files with other work. The MPL rights apply to the covered files and are not limited by this project's MIT license.

## Assets and patched upstream libraries

* Horizon vendors `wgpu-hal` 30.0.1 by the gfx-rs developers, with an upstream fix, under MIT OR Apache-2.0. Both texts are in `licenses/LICENSE-wgpu-hal-*`; corresponding source is included in the upstream Horizon tag.
* Pixelify Sans by The Pixelify Sans Project Authors is under OFL 1.1: `licenses/OFL-PixelifySans.txt`.
* Geist by its respective authors is under OFL 1.1: `licenses/OFL-Geist.txt`.
* Pixelarticons by Gerrit Halfmann are under MIT: `licenses/LICENSE-pixelarticons.txt`.

The project does **not** redistribute Minecraft assets, Minecraft JARs, OptiFine or the original Impact client. They are not covered by this project's MIT license.
