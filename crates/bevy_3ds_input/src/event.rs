use bevy::reflect::Reflect;
use bevy::ecs::event::Event;
use crate::axis::_3dsAxisType;
use crate::button::_3dsButton;
use bevy::input::ButtonState;
/// 3ds event for when the "value" on the axis changes
/// by an amount larger than the threshold defined in [`_3dsInputSettings`].
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
#[reflect(Debug, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct _3dsAxisChangedEvent {
    /// The type of the triggered axis.
    pub axis_type: _3dsAxisType,
    /// The value of the axis.
    pub value: f32,
}

impl _3dsAxisChangedEvent {
    /// Creates a [`_3dsAxisChangedEvent`].
    pub fn new(axis_type: _3dsAxisType, value: f32) -> Self {
        Self {
            axis_type,
            value,
        }
    }
}


/// A 3ds button input event, that only gets sent when the button state has changed.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Debug, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct _3dsButtonChangedEvent {
    /// The 3ds button assigned to the event.
    pub button: _3dsButton,
    /// The pressed state of the button.
    pub state: ButtonState,
}


/// A 3ds event.
///
/// This event type is used over the
/// [`_3dsButtonChangedEvent`] and [`_3dsAxisChangedEvent`] when
/// the in-frame relative ordering of events is important.
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
#[reflect(Debug, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum _3dsEvent {
    /// A button of the 3ds has been triggered.
    Button(_3dsButtonChangedEvent),
    /// An axis of the 3ds has been triggered.
    Axis(_3dsAxisChangedEvent),
}

impl From<_3dsButtonChangedEvent> for _3dsEvent {
    fn from(value: _3dsButtonChangedEvent) -> Self {
        Self::Button(value)
    }
}

impl From<_3dsAxisChangedEvent> for _3dsEvent {
    fn from(value: _3dsAxisChangedEvent) -> Self {
        Self::Axis(value)
    }
}
