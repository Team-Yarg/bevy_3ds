use bevy::input::ButtonState;
use bevy::reflect::Reflect;
use bevy::ecs::event::Event;
use bevy::ecs::system::Resource;

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


/// A 3ds button input event.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Debug, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct _3dsButtonInput {
    /// The 3ds button assigned to the event.
    pub button: _3dsButton,
    /// The pressed state of the button.
    pub state: ButtonState,
}
