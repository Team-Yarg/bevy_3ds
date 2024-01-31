use bevy::reflect::Reflect;
use bevy::ecs::event::Event;
use bevy::ecs::system::Resource;

/// An axis for a stick on the 3ds
/// ## Usage
///
/// It is used as the generic `T` value of an [`Axis`] to create `bevy` resources. These
/// resources store the data of the axes of a 3ds and can be accessed inside of a system.
///
/// ## Updating
///
/// The 3ds axes resources are updated inside of the [`_3ds_axis_event_system`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Hash, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct _3dsAxis {
    /// The type of the axis.
    pub axis_type: _3dsAxisType,
}

impl _3dsAxis {
    /// Creates a new [`_3dsAxis`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_3ds::input::axis::{_3dsAxis, _3dsAxisType};
    /// #
    /// let 3ds_axis = _3dsAxis::new(
    ///     _3dsAxisType::LeftStickX,
    /// );
    /// ```
    pub fn new(axis_type: _3dsAxisType) -> Self {
        Self { axis_type }
    }
}


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
