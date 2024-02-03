use bevy::reflect::Reflect;
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
/// The 3ds button resources are updated inside of the [`_3ds_button_event_system`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Hash, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct _3dsButton {
    /// The type of the button.
    pub button_type: _3dsButtonType,
}

impl _3dsButton {
    /// Creates a new [`_3dsButton`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_3ds::input::button::{_3dsButton, _3dsButtonType};
    /// #
    /// let 3ds_button = _3dsButton::new(
    ///     _3dsButtonType::South,
    /// );
    /// ```
    pub fn new(button_type: _3dsButtonType) -> Self {
        Self { button_type }
    }
}

/// A type of a [`_3dsButton`].
///
/// ## Usage
///
/// This is used in [`_3dsButton`] which in turn is used to create the [`Input<_3dsButton>`]
#[derive(EnumString, Display, Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Hash, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum _3dsButtonType {
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
    Null,
}
