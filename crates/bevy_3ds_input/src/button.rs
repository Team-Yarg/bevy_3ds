use bevy::reflect::Reflect;

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
        Self {
            button_type,
        }
    }
}


/// A type of a [`_3dsButton`].
///
/// ## Usage
///
/// This is used in [`_3dsButton`] which in turn is used to create the [`Input<_3dsButton>`]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
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

    SELECT,
    START,

    /// The right button of the DPAD.
    DPAD_RIGHT,
    /// The left button of the DPAD.
    DPAD_LEFT,
    /// The up button of the DPAD.
    DPAD_UP,
    /// The down button of the DPAD.
    DPAD_DOWN,


    /// The right button of the CPAD.
    CPAD_RIGHT,
    /// The left button of the CPAD.
    CPAD_LEFT,
    /// The up button of the CPAD.
    CPAD_UP,
    /// The down button of the CPAD.
    CPAD_DOWN,

    /// The right button of the CSTICK.
    CSTICK_RIGHT,
    /// The left button of the CSTICK.
    CSTICK_LEFT,
    /// The up button of the CSTICK.
    CSTICK_UP,
    /// The down button of the CSTICK.
    CSTICK_DOWN,

    /// The ZL button.
    ZL,
    /// The ZR button.
    ZR,

    /// The L button.
    L,
    /// The R button.
    R,
    NULL,

}

impl _3dsButtonType {
    pub fn to_string(&self) -> &str {
        match self {
            _3dsButtonType::B => "B",
            _3dsButtonType::A => "A",
            _3dsButtonType::Y => "Y",
            _3dsButtonType::X => "X",
            _3dsButtonType::SELECT => "SELECT",
            _3dsButtonType::START => "START",
            _3dsButtonType::DPAD_RIGHT => "DPAD_RIGHT",
            _3dsButtonType::DPAD_LEFT => "DPAD_LEFT",
            _3dsButtonType::DPAD_UP => "DPAD_UP",
            _3dsButtonType::DPAD_DOWN => "DPAD_DOWN",
            _3dsButtonType::CPAD_RIGHT => "CPAD_RIGHT",
            _3dsButtonType::CPAD_LEFT => "CPAD_LEFT",
            _3dsButtonType::CPAD_UP => "CPAD_UP",
            _3dsButtonType::CPAD_DOWN => "CPAD_DOWN",
            _3dsButtonType::CSTICK_RIGHT => "CSTICK_RIGHT",
            _3dsButtonType::CSTICK_LEFT => "CSTICK_LEFT",
            _3dsButtonType::CSTICK_UP => "CSTICK_UP",
            _3dsButtonType::CSTICK_DOWN => "CSTICK_DOWN",
            _3dsButtonType::ZL => "ZL",
            _3dsButtonType::ZR => "ZR",
            _3dsButtonType::L => "L",
            _3dsButtonType::R => "R",
            _3dsButtonType::NULL => "NULL",
        }
    }
}
