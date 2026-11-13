# Releasing a mod

Each mod ships independently. Set its version in `mod.toml` and any manifests the mod uses, and update its README with tested Horizon versions and platforms. Use a Git tag named `<slug>-v<version>`, such as `horizon-acoustics-v0.1.0`.

Build from the tagged source, run the mod’s checks, and smoke-test the package the same way a player installs it. Create a GitHub release with the packaged builds, SHA-256 checksums, install instructions, and known limitations. Mark experimental builds as prereleases. Attach new builds to GitHub Releases, keeping generated binaries and prepared Minecraft files out of the source repository.

If a published build needs a fix, release a new version so users can identify exactly which build they have.
