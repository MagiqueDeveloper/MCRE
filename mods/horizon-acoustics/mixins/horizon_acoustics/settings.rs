fn acoustics_settings(game: &crate::game::Game<'_>, id: &str, buttons: &mut Vec<String>) {
    if id == crate::game::acoustics::ID {
        let (occlusion, reverb) = crate::game::acoustics::settings(game);
        buttons.push(format!(
            "Wall muffling: {}",
            if occlusion { "On" } else { "Off" }
        ));
        buttons.push(format!(
            "Room reverb: {}",
            if reverb { "On" } else { "Off" }
        ));
    }
}
