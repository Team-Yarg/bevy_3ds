use bevy::reflect::Reflect;
use ctru::services::hid::KeyPad;
use strum_macros::Display;
use strum_macros::EnumString;
/// A button of a 3ds.
///
/// ## Usage
///
/// It is used as the generic `T` value of an [`Input`] and [`Axis`] to create `bevy` resources. These
/// resources store the data of the buttons of a 3ds and can be accessed inside of a system.
///
/// ## Updating
///
/// The 3ds button resources are updated inside of the [`button_3ds_event_system`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Hash, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct Button3ds {
    /// The type of the button.
    pub button_type: Button3dsType,
}

impl Button3ds {
    /// Creates a new [`Button3ds`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_3ds::input::button::{Button3ds, Button3dsType};
    /// #
    /// let 3ds_button = Button3ds::new(
    ///     Button3dsType::South,
    /// );
    /// ```
    pub fn new(button_type: Button3dsType) -> Self {
        Self { button_type }
    }
}

/// A type of a [`Button3ds`].
///
/// ## Usage
///
/// This is used in [`Button3ds`] which in turn is used to create the [`Input<Button3ds>`]
#[derive(EnumString, Display, Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Hash, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum Button3dsType {
    /// The bottom action button of the action pad
    B,
    /// The right action button of the action pad
    A,
    /// The upper action button of the action pad
    Y,
    /// The left action button of the action pad
    X,

    Select,
    Start,

    /// The right button of the DPAD.
    DPadRight,
    /// The left button of the DPAD.
    DPadLeft,
    /// The up button of the DPAD.
    DPadUp,
    /// The down button of the DPAD.
    DPadDown,

    /// The right button of the CPAD.
    CPadRight,
    /// The left button of the CPAD.
    CPadLeft,
    /// The up button of the CPAD.
    CPadUp,
    /// The down button of the CPAD.
    CPadDown,

    /// The right button of the CSTICK.
    CStickRight,
    /// The left button of the CSTICK.
    CStickLeft,
    /// The up button of the CSTICK.
    CStickUp,
    /// The down button of the CSTICK.
    CStickDown,

    /// The ZL button.
    ZL,
    /// The ZR button.
    ZR,

    /// The L button.
    L,
    /// The R button.
    R,
}

use std::convert::TryFrom;

impl TryFrom<KeyPad> for Button3dsType {
    type Error = ();

    fn try_from(key: KeyPad) -> Result<Self, Self::Error> {
        match key {
            KeyPad::B => Ok(Button3dsType::B),
            KeyPad::A => Ok(Button3dsType::A),
            KeyPad::Y => Ok(Button3dsType::Y),
            KeyPad::X => Ok(Button3dsType::X),
            KeyPad::SELECT => Ok(Button3dsType::Select),
            KeyPad::START => Ok(Button3dsType::Start),
            KeyPad::DPAD_RIGHT => Ok(Button3dsType::DPadRight),
            KeyPad::DPAD_LEFT => Ok(Button3dsType::DPadLeft),
            KeyPad::DPAD_UP => Ok(Button3dsType::DPadUp),
            KeyPad::DPAD_DOWN => Ok(Button3dsType::DPadDown),
            KeyPad::CPAD_RIGHT => Ok(Button3dsType::CPadRight),
            KeyPad::CPAD_LEFT => Ok(Button3dsType::CPadLeft),
            KeyPad::CPAD_UP => Ok(Button3dsType::CPadUp),
            KeyPad::CPAD_DOWN => Ok(Button3dsType::CPadDown),
            KeyPad::CSTICK_RIGHT => Ok(Button3dsType::CStickRight),
            KeyPad::CSTICK_LEFT => Ok(Button3dsType::CStickLeft),
            KeyPad::CSTICK_UP => Ok(Button3dsType::CStickUp),
            KeyPad::CSTICK_DOWN => Ok(Button3dsType::CStickDown),
            KeyPad::ZL => Ok(Button3dsType::ZL),
            KeyPad::ZR => Ok(Button3dsType::ZR),
            KeyPad::L => Ok(Button3dsType::L),
            KeyPad::R => Ok(Button3dsType::R),
            _ => Err(()),
        }
    }
}
