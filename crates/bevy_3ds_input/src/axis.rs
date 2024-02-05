use bevy::reflect::Reflect;

/// An axis for a stick on the 3ds
/// ## Usage
///
/// It is used as the generic `T` value of an [`Axis`] to create `bevy` resources. These
/// resources store the data of the axes of a 3ds and can be accessed inside of a system.
///
/// ## Updating
///
/// The 3ds axes resources are updated inside of the [`axis_3ds_event_system`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Hash, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct Axis3ds {
    /// The type of the axis.
    pub axis_type: Axis3dsType,
}

impl Axis3ds {
    /// Creates a new [`Axis3ds`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_3ds::input::axis::{Axis3ds, Axis3dsType};
    /// #
    /// let 3ds_axis = Axis3ds::new(
    ///     Axis3dsType::LeftStickX,
    /// );
    /// ```
    pub fn new(axis_type: Axis3dsType) -> Self {
        Self { axis_type }
    }
}


/// A type of a [`Axis3ds`].
///
/// ## Usage
///
/// This is used to determine which axis has changed its value when receiving a
/// [`Axis3dsChangedEvent`]. It is also used in the [`Axis3ds`]
/// which in turn is used to create the [`Axis<Axis3ds>`] `bevy` resource.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Debug, Hash, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum Axis3dsType {
    /// The horizontal value of the left CPAD.
    CPadX,
    /// The vertical value of the left CPAD.
    CPadY,

    /// The horizontal value of the right CSTICK.
    CStickX,
    /// The vertical value of the right CSTICK.
    CstickY,

    // volume slider
    Volume,
}
