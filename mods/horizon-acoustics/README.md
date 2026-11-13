# Horizon Acoustics 0.1.0

A Rust implementation of two Sound Physics Remastered features for Horizon 26.3, based on the original mod’s public feature description. Wall muffling reduces the volume and high frequencies of nearby positional sounds. Room reverb adds a short stereo tail that is stronger in enclosed spaces with reflective blocks. Both have switches in **Options → Mods → Horizon Acoustics → Settings**.

This prototype applies to mono positional game sounds. Its room model uses six short rays and material estimates. Voice chat, non-positional sounds, stereo music and split-screen are outside this release. It has no Java dependency and uses Horizon's existing audio output.

## Try the Apple Silicon build

Download the [Apple Silicon preview](https://github.com/MagiqueDeveloper/MCRE/releases/tag/horizon-acoustics-v0.1.0). In Horizon Launcher, open **Versions → Import a game build…**, select the ZIP, and use Nightly `nightly-2026-11-08-fdc87ae` as the base. In the instance’s version picker next to **Play**, select the imported build and launch it. The launcher uses the prepared Minecraft resources from that installation. Run a test world, play a note block nearby, then compare it through a stone wall and from inside a closed stone room. The settings switches let you compare each effect in the same world.

## Build from source

This version requires the host patch in this directory and is pinned to Horizon `fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca` and horizon-mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`. The stock launcher's mixin tool rejects the audio callback's conditional attribute, so building this version requires the host patch. The host patch connects the effect to the mixer and the camera's listener position.

From a clean checkout of the pinned Horizon commit, with the pinned mixin executable and a Rust toolchain on your PATH:

```sh
git apply /path/to/horizon-acoustics/audio-host.patch
horizon-mixins apply --workspace "$PWD" \
  --mods /path/to/horizon-acoustics/mixins \
  --out /path/to/new/staged-horizon
cd /path/to/new/staged-horizon
cargo build --locked --profile fast -p horizon-client --bin horizon-client
cargo test --locked --profile fast -p horizon-client-render --features audio --lib audio::
```

The build output is `target/fast/horizon-client`. If you run it directly, pass `--resources` with your prepared Horizon 26.3 resources and use a separate `--game-dir` for the test world. Keep the included `native-mods.json` beside the executable so the game recognizes the build as modded.

## Source

The Rust implementation was written from the public description of [Sound Physics Remastered](https://modrinth.com/mod/sound-physics-remastered), by henkelmax and its upstream authors. See [Horizon](https://github.com/horizon-rust/horizon) and the included MIT license for the host patch.
