# macOS Setup

Short guide. Apple Silicon Mac tested.

## 1. Install Apple tools

```sh
xcode-select --install
```

Already installed? Good. Move on.

## 2. Install Homebrew

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Add Brew to shell when installer tells you. Apple Silicon default:

```sh
eval "$(/opt/homebrew/bin/brew shellenv)"
```

## 3. Install native tools

```sh
brew install llvm cmake imagemagick
```

- LLVM builds PSP code.
- CMake builds PPSSPP.
- ImageMagick checks emulator screenshots.

## 4. Install Rust

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
source "$HOME/.cargo/env"
rustup component add rustfmt --toolchain stable
```

## 5. Install Bun

```sh
curl -fsSL https://bun.com/install | bash
export PATH="$HOME/.bun/bin:$PATH"
```

## 6. Get project

New clone:

```sh
git clone --recursive https://github.com/CMeech/zombies-psp.git
cd zombies-psp
```

Existing clone:

```sh
git submodule update --init --recursive
```

Install project dependencies. Install pinned PSP tools and SDK:

```sh
export PATH="/opt/homebrew/bin:$HOME/.bun/bin:$HOME/.cargo/bin:/opt/homebrew/opt/llvm/bin:$PATH"
export POCKETJS_LLVM_BIN="/opt/homebrew/opt/llvm/bin"

bun run setup
bun run bootstrap
```

Bootstrap takes time. It downloads Rust nightly, `cargo-psp`, and PSP SDK.

## 7. Build PPSSPPHeadless

Use separate folder beside project:

```sh
cd ..
git clone https://github.com/hrydgard/ppsspp.git
cd ppsspp
git submodule update --init --recursive
mkdir Build
cd Build
cmake -DHEADLESS=ON ..
make -j"$(sysctl -n hw.ncpu)"
```

Tell project where binary lives. Use your real path:

```sh
export PPSSPP_HEADLESS="/absolute/path/to/ppsspp/Build/PPSSPPHeadless"
```

Test it:

```sh
"$PPSSPP_HEADLESS" --version
```

## 8. Add temporary local map

Never commit BSP, WAD, cooked map, or map package.

```text
zombies-psp/local/openstrike-maps/
├── maps/
│   └── de_dust2.bsp
└── support/
    └── cs_dust.wad
```

From project root:

```sh
export OPENSTRIKE_MAPS="$PWD/local/openstrike-maps"
```

This upstream map is temporary. Original project maps replace it later.

## 9. Build game for macOS

From project root:

```sh
export PATH="/opt/homebrew/bin:$HOME/.bun/bin:$HOME/.cargo/bin:/opt/homebrew/opt/llvm/bin:$PATH"
export POCKETJS_LLVM_BIN="/opt/homebrew/opt/llvm/bin"
export OPENSTRIKE_MAPS="$PWD/local/openstrike-maps"

bun run build:ui
cargo build --release -p openstrike
```

Run native game:

```sh
target/release/openstrike --maps-dir "$OPENSTRIKE_MAPS"
```

Controls:

- Mouse look.
- WASD move.
- Left click fire.
- R reload.
- Space jump.
- Escape release mouse.
- F3 debug panel.

Five-second smoke test:

```sh
target/release/openstrike --maps-dir "$OPENSTRIKE_MAPS" --auto-quit 5
```

Headless tests:

```sh
target/release/openstrike --maps-dir "$OPENSTRIKE_MAPS" --script walk --screenshot out/walk
target/release/openstrike --maps-dir "$OPENSTRIKE_MAPS" --script round --screenshot out/round
```

## 10. Build PSP game

```sh
bun scripts/psp.ts --release --package
```

Output:

```text
dist/PSP/GAME/OpenStrike/
```

Run emulator journey:

```sh
bun scripts/e2e-psp.ts
```

Current PPSSPP can be newer than upstream golden version. All frames may run but byte comparison may differ. Fine for compatibility smoke. Make new goldens after original maps exist.

## New terminal?

Exports disappear when terminal closes. Run again:

```sh
cd /absolute/path/to/zombies-psp
export PATH="/opt/homebrew/bin:$HOME/.bun/bin:$HOME/.cargo/bin:/opt/homebrew/opt/llvm/bin:$PATH"
export POCKETJS_LLVM_BIN="/opt/homebrew/opt/llvm/bin"
export PPSSPP_HEADLESS="/absolute/path/to/ppsspp/Build/PPSSPPHeadless"
export OPENSTRIKE_MAPS="$PWD/local/openstrike-maps"
```
