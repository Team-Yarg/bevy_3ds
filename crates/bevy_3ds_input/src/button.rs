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
