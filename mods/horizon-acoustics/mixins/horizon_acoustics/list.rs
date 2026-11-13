fn list_acoustics(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    mods.push(crate::game::native_hooks::NativeMod {
        id: crate::game::acoustics::ID.into(),
        title: "Horizon Acoustics 0.1.0".into(),
        description:
            "Muffles positional sounds through walls and adds reflections in enclosed spaces."
                .into(),
        keys: Vec::new(),
    });
}
