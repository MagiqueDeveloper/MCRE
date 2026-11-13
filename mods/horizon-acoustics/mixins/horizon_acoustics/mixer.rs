use super::{Audio, ChannelId, State};
use std::collections::HashMap;
use std::time::{Duration, Instant};

const MAX_DISTANCE: f64 = 48.0;
const MAX_SOURCES: usize = 64;
const MAX_RAY_VISITS: usize = 96;
const REFRESH: Duration = Duration::from_millis(50);

#[derive(Clone, Copy, Default)]
pub struct Material {
    pub absorption: f32,
    pub reflection: f32,
}

pub fn material(name: &str, solid: bool, water: bool) -> Material {
    if water {
        return Material {
            absorption: 0.10,
            reflection: 0.0,
        };
    }
    if !solid {
        return Material::default();
    }
    let (absorption, reflection) = if name.contains("wool") || name.contains("sponge") {
        (0.85, 0.12)
    } else if name.contains("glass") {
        (0.38, 0.72)
    } else if name.contains("planks") || name.ends_with("_log") || name.ends_with("_wood") {
        (0.55, 0.52)
    } else if name.contains("dirt") || name.contains("sand") || name.contains("grass_block") {
        (0.65, 0.42)
    } else {
        (0.75, 0.88)
    };
    Material {
        absorption,
        reflection,
    }
}

fn trace<T>(
    from: [f64; 3],
    to: [f64; 3],
    mut visit: impl FnMut([i32; 3]) -> Option<T>,
) -> Option<T> {
    use horizon_core::geometry::{Vec3, traverse_blocks};
    let from = Vec3::new(from[0], from[1], from[2]);
    let to = Vec3::new(to[0], to[1], to[2]);
    traverse_blocks(from, to, MAX_RAY_VISITS, |p| visit([p.x, p.y, p.z]))
        .ok()
        .flatten()
}

fn obstruction(
    from: [f64; 3],
    to: [f64; 3],
    sample: &impl Fn([i32; 3]) -> Option<Material>,
) -> f32 {
    let endpoint = |point: [f64; 3]| point.map(|x| x.floor() as i32);
    let (start, end) = (endpoint(from), endpoint(to));
    let mut depth: f32 = 0.0;
    trace(from, to, |pos| {
        if pos != start && pos != end {
            let Some(material) = sample(pos) else {
                return Some(());
            };
            depth += material.absorption;
        }
        (depth >= 3.0).then_some(())
    });
    depth.min(3.0)
}

#[derive(Clone, Copy, Default)]
struct Room {
    wet: f32,
    feedback: f32,
}

fn room(at: [f64; 3], sample: &impl Fn([i32; 3]) -> Option<Material>) -> Room {
    let start = at.map(|x| x.floor() as i32);
    let mut vertical = [false; 2];
    let mut walls = 0;
    let mut reflectivity = 0.0;
    for axis in 0..3 {
        for (side, step) in [-16.0, 16.0].into_iter().enumerate() {
            let mut to = at;
            to[axis] += step;
            if let Some(Some(material)) = trace(at, to, |pos| {
                if pos == start {
                    return None;
                }
                match sample(pos) {
                    None => Some(None),
                    Some(value) if value.reflection > 0.0 => Some(Some(value)),
                    _ => None,
                }
            }) {
                walls += 1;
                reflectivity += material.reflection;
                if axis == 1 {
                    vertical[side] = true;
                }
            }
        }
    }
    if vertical != [true, true] || walls < 4 {
        return Room::default();
    }
    Room {
        wet: (reflectivity / 6.0) * 0.36,
        feedback: 0.27 + (reflectivity / 6.0) * 0.42,
    }
}

#[derive(Default)]
pub(super) struct Voice {
    depth: f32,
    low: f32,
    gain: f32,
    cutoff: f32,
}

impl Voice {
    fn new() -> Self {
        Self {
            gain: 1.0,
            cutoff: 1.0,
            ..Self::default()
        }
    }

    fn tune(&mut self, rate: u32, frames: usize) {
        let hz = 18_000.0 * (-1.25 * self.depth).exp();
        let blend = 1.0 - (-(frames as f32) / (rate as f32 * 0.025)).exp();
        let target = if self.depth == 0.0 {
            1.0
        } else {
            1.0 - (-std::f32::consts::TAU * hz / rate as f32).exp()
        };
        self.cutoff += (target - self.cutoff) * blend;
        self.gain += ((-0.61 * self.depth).exp() - self.gain) * blend;
    }

    pub(super) fn sample(&mut self, input: f32) -> f32 {
        self.low += self.cutoff * (input - self.low);
        self.low * self.gain
    }
}

#[derive(Default)]
struct Delay {
    samples: Vec<f32>,
    cursor: usize,
    damped: f32,
}

impl Delay {
    fn resize(&mut self, frames: usize) {
        self.samples = vec![0.0; frames.max(1)];
        self.cursor = 0;
        self.damped = 0.0;
    }

    fn next(&mut self, input: f32, feedback: f32) -> f32 {
        let delayed = self.samples[self.cursor];
        self.damped += 0.36 * (delayed - self.damped);
        self.samples[self.cursor] = input + self.damped * feedback;
        self.cursor += 1;
        if self.cursor == self.samples.len() {
            self.cursor = 0;
        }
        delayed
    }
}

#[derive(Default)]
pub(super) struct Effects {
    voices: HashMap<ChannelId, Voice>,
    send: Vec<f32>,
    delays: [Delay; 4],
    rate: u32,
    room: Room,
    wet: f32,
    next_update: Option<Instant>,
    world: u32,
}

impl Effects {
    pub(super) fn begin(&mut self, frames: usize, rate: u32) {
        if self.rate != rate {
            self.rate = rate;
            for (delay, seconds) in self.delays.iter_mut().zip([0.031, 0.043, 0.053, 0.067]) {
                delay.resize((rate as f32 * seconds) as usize);
            }
        }
        self.send.resize(frames, 0.0);
        self.send.fill(0.0);
        for voice in self.voices.values_mut() {
            voice.tune(rate, frames);
        }
    }

    pub(super) fn voice(&mut self, id: ChannelId) -> Option<(&mut Voice, &mut [f32])> {
        Some((self.voices.get_mut(&id)?, &mut self.send))
    }

    pub(super) fn reverb(&mut self, output: &mut [f32], channels: usize) {
        let blend = 1.0 - (-1.0 / (self.rate as f32 * 0.05)).exp();
        for (input, frame) in self.send.iter().zip(output.chunks_mut(channels)) {
            self.wet += (self.room.wet - self.wet) * blend;
            let [a, b, c, d] = &mut self.delays;
            let input = input * self.wet;
            let left = a.next(input, self.room.feedback) + c.next(input, self.room.feedback);
            let right = b.next(input, self.room.feedback) + d.next(input, self.room.feedback);
            if channels >= 2 {
                frame[0] += left * 0.5;
                frame[1] += right * 0.5;
            } else {
                frame[0] += (left + right) * 0.25;
            }
        }
    }
}

pub fn update(
    audio: &Audio,
    occlusion: bool,
    reverb: bool,
    sample: impl Fn([i32; 3]) -> Option<Material>,
) {
    if audio.backend.owner != 0 || !audio.backend.live {
        return;
    }
    let now = Instant::now();
    let world = audio.backend.world;
    let Ok(mut mixer) = audio.backend.mixer.try_lock() else {
        return;
    };
    if !(occlusion || reverb) || !mixer.listeners.is_empty() {
        mixer.acoustics_effects = None;
        return;
    }
    let at = mixer.listener;
    let effects = mixer.acoustics_effects.get_or_insert_with(Box::default);
    if effects.world != world {
        **effects = Effects {
            world,
            ..Effects::default()
        };
    }
    if effects.next_update.is_some_and(|next| now < next) {
        return;
    }
    effects.next_update = Some(now + REFRESH);
    let mut nearby = Vec::new();
    for (&id, channel) in &mixer.channels {
        let Some((_, _, 1)) = &channel.samples else {
            continue;
        };
        if channel.state != State::Playing
            || channel.relative
            || channel.linear.is_none()
            || channel.world != world
        {
            continue;
        }
        let position = channel.at.map(f64::from);
        let distance: f64 = position.iter().zip(at).map(|(a, b)| (a - b).powi(2)).sum();
        let range = f64::from(channel.linear.unwrap_or(0.0)).min(MAX_DISTANCE);
        if distance < range * range {
            nearby.push((id, position, distance));
        }
    }
    drop(mixer);

    nearby.sort_unstable_by(|a, b| a.2.total_cmp(&b.2).then(a.0.cmp(&b.0)));
    nearby.truncate(MAX_SOURCES);
    let depths: HashMap<_, _> = nearby
        .into_iter()
        .map(|(id, position, _)| {
            let depth = if occlusion {
                obstruction(at, position, &sample)
            } else {
                0.0
            };
            (id, depth)
        })
        .collect();
    let room = if reverb {
        room(at, &sample)
    } else {
        Room::default()
    };

    let Ok(mut mixer) = audio.backend.mixer.try_lock() else {
        return;
    };
    if let Some(effects) = mixer
        .acoustics_effects
        .as_mut()
        .filter(|effects| effects.world == world)
    {
        effects.voices.retain(|id, _| depths.contains_key(id));
        for (id, depth) in depths {
            effects.voices.entry(id).or_insert_with(Voice::new).depth = depth;
        }
        effects.room = room;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::{Channel, Mixer, State, mix};
    use std::sync::Arc;

    #[test]
    fn a_wall_absorbs_sound_and_open_air_does_not() {
        let source = [4.5, 0.5, 0.5];
        let listener = [0.5, 0.5, 0.5];
        let sample = |[x, _, _]: [i32; 3]| {
            Some(if x == 2 {
                material("minecraft:stone", true, false)
            } else {
                Material::default()
            })
        };
        assert_eq!(obstruction(listener, source, &sample), 0.75);
        assert_eq!(obstruction(source, listener, &sample), 0.75);
        assert_eq!(
            obstruction(listener, source, &|_| Some(Material::default())),
            0.0
        );
    }

    #[test]
    fn reverb_requires_an_enclosure_and_respects_surface_reflection() {
        let listener = [0.5; 3];
        let enclosed = |name| {
            room(listener, &|position| {
                let wall = position.into_iter().any(|axis: i32| axis.abs() >= 3);
                Some(material(name, wall, false))
            })
        };
        let outdoors = room(listener, &|[_, y, _]| {
            Some(material("minecraft:stone", y < 0, false))
        });
        assert_eq!(outdoors.wet, 0.0);
        assert!(enclosed("minecraft:stone").wet > enclosed("minecraft:white_wool").wet);
        assert!(enclosed("minecraft:white_wool").wet > 0.0);
    }

    fn fixture(samples: Vec<f32>, relative: bool) -> Mixer {
        let mut mixer = Mixer::default();
        mixer.channels.insert(
            1,
            Channel {
                samples: Some((Arc::new(samples), 48_000, 1)),
                position: 0.0,
                state: State::Playing,
                gain: 1.0,
                pitch: 1.0,
                linear: Some(16.0),
                looping: false,
                at: [0.0, 0.0, 2.0],
                relative,
                owner: 0,
                world: 0,
                started: None,
            },
        );
        mixer
    }

    fn with_effects(mixer: &mut Mixer, depth: f32, room: Room) {
        let mut voice = Voice::new();
        voice.depth = depth;
        mixer.acoustics_effects = Some(Box::new(Effects {
            voices: HashMap::from([(1, voice)]),
            room,
            ..Effects::default()
        }));
    }

    #[test]
    fn the_mixer_muffles_high_frequencies_and_leaves_relative_audio_alone() {
        let input: Vec<_> = (0..4097)
            .map(|i| if i % 2 == 0 { 0.3 } else { -0.3 })
            .collect();
        let render = |relative, filtered| {
            let mut mixer = fixture(input.clone(), relative);
            if filtered {
                with_effects(&mut mixer, 2.0, Room::default());
            }
            let mut output = [0.0; 4096];
            mix(&mut mixer, &mut output, 1, 48_000);
            output
        };
        let dry = render(false, false);
        let occluded = render(false, true);
        assert!(occluded[100].abs() < dry[100].abs() * 0.25);
        assert_eq!(render(true, true), render(true, false));
    }

    #[test]
    fn room_reflections_reach_the_output_after_the_direct_sound() {
        let mut input = vec![0.0; 4097];
        input[32] = 0.5;
        let mut dry = fixture(input.clone(), false);
        let mut wet = fixture(input, false);
        with_effects(
            &mut wet,
            0.0,
            Room {
                wet: 0.3,
                feedback: 0.55,
            },
        );
        let mut baseline = [0.0; 8192];
        let mut reflected = [0.0; 8192];
        mix(&mut dry, &mut baseline, 2, 48_000);
        mix(&mut wet, &mut reflected, 2, 48_000);
        assert_eq!(baseline[..2048], reflected[..2048]);
        assert!(baseline[2048..].iter().all(|sample| *sample == 0.0));
        assert!(reflected[2048..].iter().any(|sample| sample.abs() > 0.0001));
    }
}
