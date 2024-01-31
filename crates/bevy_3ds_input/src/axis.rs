use bevy::reflect::Reflect;

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


/// A type of a [`_3dsAxis`].
///
/// ## Usage
///
/// This is used to determine which axis has changed its value when receiving a
/// [`_3dsAxisChangedEvent`]. It is also used in the [`_3dsAxis`]
/// which in turn is used to create the [`Axis<_3dsAxis>`] `bevy` resource.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Hash, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum _3dsAxisType {
    /// The horizontal value of the left stick.
    LeftStickX,
    /// The vertical value of the left stick.
    LeftStickY,

    /// The horizontal value of the right stick.
    RightStickX,
    /// The vertical value of the right stick.
    RightStickY,
}
