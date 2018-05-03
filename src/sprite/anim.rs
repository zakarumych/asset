
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Range;
use std::time::Duration;

#[cfg(feature="serde")]
use serde::{Serialize, de::DeserializeOwned};

#[cfg(feature="serde")]
use asset::{Asset, SerdeLoader};

use sprite::{Rect, SpriteSheet};

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature="serde", serde(bound(serialize = "I: Serialize + Eq + Hash", deserialize = "I: DeserializeOwned + Eq + Hash")))]
#[derive(Derivative)]
#[derivative(Clone, Debug(bound = "I: Eq + Hash + Debug"), PartialEq(bound = "I: Eq + Hash"), Eq(bound = "I: Eq + Hash"))]
pub struct AnimationSet<I> {
    /// Maps name to frames range.
    animations: HashMap<I, Range<u32>>,
}

#[cfg(feature="serde")]
impl<I> Asset for AnimationSet<I>
where
    I: DeserializeOwned + Eq + Hash + Send + Sync + 'static,
{
    type Loader = SerdeLoader;

    const KIND: &'static str = "AnimationSet";
}

impl<I> AnimationSet<I>
where
    I: Hash + Eq,
{
    /// Create new animation set.
    pub fn new() -> Self {
        AnimationSet {
            animations: HashMap::new(),
        }
    }

    /// Add animations.
    pub fn add_animation(&mut self, id: I, frames: Range<u32>) -> &mut Self {
        self.animations.insert(id, frames);
        self
    }

    /// Add animations.
    pub fn with_animation(mut self, id: I, frames: Range<u32>) -> Self {
        self.add_animation(id, frames);
        self
    }

    /// Get frames range.
    pub fn get(&self, id: &I) -> Option<Range<u32>> {
        self.animations.get(id).cloned()
    }
}

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Animation<I> {
    elapsed: u64,
    range: Range<u32>,
    repeat: Option<u64>,
    next: Option<(I, Option<u64>)>,
}

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
enum Current<I> {
    Animated(Animation<I>),
    Still(u32),
}

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature="serde", serde(bound(serialize = "I: Serialize + Eq + Hash", deserialize = "I: DeserializeOwned + Eq + Hash")))]
#[derive(Derivative)]
#[derivative(Clone, Debug(bound = "I: Eq + Hash + Debug"), PartialEq(bound = "I: Eq + Hash"), Eq(bound = "I: Eq + Hash"))]
pub struct AnimationController<I> {
    set: AnimationSet<I>,
    layout: SpriteSheet,
    frame_duration: u64,
    current: Current<I>,
}

#[cfg(feature="serde")]
impl<I> Asset for AnimationController<I>
where
    I: DeserializeOwned + Eq + Hash + Send + Sync + 'static,
{
    type Loader = SerdeLoader;

    const KIND: &'static str = "AnimationController";
}

impl<I> AnimationController<I>
where
    I: Hash + Eq,
{
    /// Create new animation controller.
    pub fn new(set: AnimationSet<I>, layout: SpriteSheet, duration: Duration) -> Self {
        AnimationController {
            set,
            layout,
            frame_duration: duration_to_nanos(&duration),
            current: Current::Still(0),
        }
    }

    pub fn sample(&self) -> Rect {
        let frame = match self.current {
            Current::Animated(ref animation) => {
                animation.range.start + (animation.elapsed / self.frame_duration) as u32
            },
            Current::Still(frame) => frame,
        };
        self.layout.sample(frame)
    }

    pub fn advance(&mut self, duration: Duration) {
        let mut duration = duration_to_nanos(&duration);
        while duration > 0 {
            duration = self.advance_step(duration);
        }
    }

    pub fn play(&mut self, id: I, repeat: Option<u64>, next: Option<(I, Option<u64>)>) {
        let range = self.set.get(&id).unwrap();
        self.current = Current::Animated(Animation {
            elapsed: 0,
            range,
            repeat,
            next,
        });
    }

    fn advance_step(&mut self, duration: u64) -> u64 {
        let (left, replace) = match self.current {
            Current::Still { .. } => { return 0; },
            Current::Animated(ref mut animation) => {
                animation.elapsed += duration;
                let animation_duration = self.frame_duration * animation.range.len() as u64;
                let laps = animation.elapsed / animation_duration;
                match (laps, &mut animation.repeat) {
                    (0, _) | (_, &mut None) => {
                        animation.elapsed %= animation_duration;
                        return 0;
                    }
                    (laps, &mut Some(ref mut repeat)) => {
                        if *repeat > laps {
                            *repeat -= laps;
                            return 0;
                        } else {
                            let laps_left = laps - *repeat;
                            let left = laps_left * animation_duration;
                            match animation.next {
                                None => {
                                    (0, Current::Still(animation.range.end - 1))
                                },
                                Some((ref id, ref repeat)) => {
                                    (left, Current::Animated(Animation {
                                        elapsed: 0,
                                        range: self.set.get(id).unwrap(),
                                        repeat: *repeat,
                                        next: None,
                                    }))
                                },
                            }
                        }
                    }
                }
            },
        };

        self.current = replace;
        left
    }
}


fn duration_to_nanos(duration: &Duration) -> u64 {
    let nanos = duration.subsec_nanos() as u64;
    let secs = duration.as_secs();
    secs * 1_000_000_000 + nanos
}

