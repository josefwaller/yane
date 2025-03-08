use sdl2::keyboard::Keycode;
use serde::{de::Visitor, Deserialize, Serialize};

/// Simple wrapper around [sdl2::keyboard::Keycode] that implements [serde::Deserialize] and [serde::Serialize].
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
impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match sdl2::keyboard::Scancode::from_keycode(self.code) {
            Some(s) => serializer.serialize_str(s.name()),
            None => Err(serde::ser::Error::custom(format!(
                "Unable to serialize key {:?}",
                self.code
            ))),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
/// Controls for an NES controller.
///
/// Map of SDL key codes to buttons on the controller.
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
#[derive(Serialize, Deserialize, Clone)]
/// Controls for running the emulator.
///
/// Map of SDL key codes to actions in the emulator app (press a button, pause, volume up, etc).
pub struct KeyMap {
    pub controllers: [Controller; 2],
    pub pause: Key,
    pub volume_up: Key,
    pub volume_down: Key,
    pub quicksave: Key,
    pub quickload: Key,
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
            quicksave: sdl_key!(F1),
            quickload: sdl_key!(F2),
        }
    }
}
