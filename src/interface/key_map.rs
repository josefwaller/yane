use sdl2::keyboard::{Keycode, Scancode};
use serde::{de::Visitor, Deserialize, Serialize};

// This just exists so we can implement Serialize for it
#[derive(Clone)]
pub struct Key {
    pub code: Keycode,
}
impl From<Keycode> for Key {
    fn from(code: Keycode) -> Self {
        Key { code }
    }
}

struct KeyVisitor;
impl<'de> Visitor<'de> for KeyVisitor {
    type Value = Key;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("the name of a SDL2 scancode")
    }
    fn visit_str<E>(self, name: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match sdl2::keyboard::Scancode::from_name(name) {
            Some(scancode) => match sdl2::keyboard::Keycode::from_scancode(scancode) {
                Some(code) => Ok(Key { code }),
                None => Err(E::custom(format!(
                    "Unable to get scancode from keycode {}",
                    name
                ))),
            },
            None => Err(E::custom(format!(
                "Can't parse scancode from name {}",
                name
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(KeyVisitor {})
    }
}

#[derive(Deserialize, Clone)]
pub struct Controller {
    pub a: Key,
    pub b: Key,
    pub up: Key,
    pub left: Key,
    pub right: Key,
    pub down: Key,
    pub start: Key,
    pub select: Key,
}
#[derive(Deserialize, Clone)]
pub struct KeyMap {
    pub controllers: [Controller; 2],
    pub pause: Key,
    pub volume_up: Key,
    pub volume_down: Key,
}

impl Default for KeyMap {
    fn default() -> Self {
        macro_rules! sdl_key {
            ($key: ident) => {
                Keycode::$key.into()
            };
        }
        KeyMap {
            controllers: [
                Controller {
                    a: sdl_key!(SPACE),
                    b: sdl_key!(LSHIFT),
                    up: sdl_key!(W),
                    left: sdl_key!(A),
                    right: sdl_key!(D),
                    down: sdl_key!(S),
                    start: sdl_key!(E),
                    select: sdl_key!(Q),
                },
                Controller {
                    a: sdl_key!(N),
                    b: sdl_key!(RSHIFT),
                    up: sdl_key!(I),
                    down: sdl_key!(K),
                    left: sdl_key!(J),
                    right: sdl_key!(L),
                    start: sdl_key!(U),
                    select: sdl_key!(O),
                },
            ],
            pause: sdl_key!(P),
            volume_up: sdl_key!(UP),
            volume_down: sdl_key!(DOWN),
        }
    }
}
