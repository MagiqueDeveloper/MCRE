# Contributing to MCRE

Use [Issues](https://github.com/MagiqueDeveloper/MCRE/issues/new/choose) for reproducible bugs and port proposals. Use [Discussions](https://github.com/MagiqueDeveloper/MCRE/discussions) for questions, design ideas, and early experiments.

## Working on a mod

1. Check for an existing issue or pull request. For a new port, start with a proposal so contributors can agree on the first playable version and avoid duplicating work.
2. Fork MCRE and make a branch from `main`. Keep your changes in the relevant directory under `mods/`.
3. Follow the mod’s README to build it and run its checks. In your pull request, name the Horizon build you tested, describe how you verified the behavior, and list any limitations.
4. Link relevant upstream code or documentation. If your work adapts code or assets, include the source revision, license, and required notices. Keep Minecraft files and downloaded mod binaries out of Git.

A new mod gets its own folder under `mods/<slug>/` with a README, a license or a link to the applicable repository license, and `mod.toml`. Copy the metadata fields from [Horizon Acoustics](mods/horizon-acoustics/mod.toml), add the mod to the README table, and document the checks contributors can run. The repository checks the catalog with `python3 scripts/check_repo.py` using Python 3.11 or newer.

Pull requests go through the checks on GitHub and a review from the relevant maintainer when available. Follow the [Code of Conduct](CODE_OF_CONDUCT.md). See the [release guide](docs/releases.md) when preparing a build for players.
