use super::Game;
use horizon_client::audio::acoustics::{Material, material, update as update_mixer};
use horizon_client::blocks::Fluid;
use std::cell::Cell;
use std::time::{Duration, Instant};

pub(super) const ID: &str = "horizon_acoustics";
const KEYS: [&str; 2] = ["occlusion", "reverb"];

thread_local! {
    static SETTINGS: Cell<Option<(Instant, bool, bool)>> = const { Cell::new(None) };
}

pub(super) fn settings(game: &Game<'_>) -> (bool, bool) {
    let enabled = |key| game.native_setting(ID, key).as_deref() != Some("off");
    (enabled(KEYS[0]), enabled(KEYS[1]))
}

pub(super) fn toggle(game: &Game<'_>, button: usize) {
    if let Some(key) = KEYS.get(button) {
        let (occlusion, reverb) = settings(game);
        let was_on = [occlusion, reverb][button];
        game.native_set_setting(ID, key, if was_on { "off" } else { "on" });
        SETTINGS.set(None);
    }
}

pub(super) fn update(game: &Game<'_>) {
    if !game.native_window() {
        return;
    }
    let now = Instant::now();
    let (occlusion, reverb) = match SETTINGS.get() {
        Some((next, occlusion, reverb)) if now < next => (occlusion, reverb),
        _ => {
            let (occlusion, reverb) = settings(game);
            SETTINGS.set(Some((now + Duration::from_secs(1), occlusion, reverb)));
            (occlusion, reverb)
        }
    };
    let active =
        game.title.is_none() && game.player.is_some() && !game.settings.mods.disabled.contains(ID);
    let world = &game.world;
    let facts = game.res.facts();
    update_mixer(
        &game.audio,
        active && occlusion,
        active && reverb,
        |[x, y, z]| {
            let chunk = world.chunks.get(&(x >> 4, z >> 4))?;
            let state = chunk.block(x & 15, y, z & 15);
            if world.is_air(state) {
                return Some(Material::default());
            }
            let block = facts.get(state);
            Some(material(
                &block.name,
                block.full_collision,
                block.fluid == Fluid::Water,
            ))
        },
    );
}
