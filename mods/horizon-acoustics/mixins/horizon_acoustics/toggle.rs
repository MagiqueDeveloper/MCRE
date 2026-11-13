fn toggle_acoustics(game: &crate::game::Game<'_>, id: &str, button: usize) {
    if id == crate::game::acoustics::ID {
        crate::game::acoustics::toggle(game, button);
    }
}
