use bevy::input::ButtonState;
//! The 3ds input functionality.


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
    /// # use bevy_3ds::input::_3ds::{_3dsAxis, _3dsAxisType};
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
    /// # use bevy_3ds::input::_3ds::{_3dsButton, _3dsButtonType};
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


pub struct _3dsInputSettings {
    pub default_button_settings: ButtonSettings,
    pub default_axis_settings: AxisSettings,
    pub default_button_axis_settings: ButtonAxisSettings,
    pub button_settings: HashMap<_3dsButton, ButtonSettings>,
    pub axis_settings: HashMap<_3dsAxis, AxisSettings>,
    pub button_axis_settings: HashMap<_3dsButton, ButtonAxisSettings>,
}


/// Settings for a [`_3dsAxis`].
///
/// It is used inside of the [`_3dsInputSettings`] to define the sensitivity range and
/// threshold for an axis.
/// Values that are higher than `livezone_upperbound` will be rounded up to 1.0.
/// Values that are lower than `livezone_lowerbound` will be rounded down to -1.0.
/// Values that are in-between `deadzone_lowerbound` and `deadzone_upperbound` will be rounded
/// to 0.0.
/// Otherwise, values will not be rounded.
///
/// The valid range is `[-1.0, 1.0]`.
#[derive(Debug, Clone, Reflect, PartialEq)]
#[reflect(Debug, Default)]
pub struct AxisSettings {
    /// Values that are higher than `livezone_upperbound` will be rounded up to 1.0.
    livezone_upperbound: f32,
    /// Positive values that are less than `deadzone_upperbound` will be rounded down to 0.0.
    deadzone_upperbound: f32,
    /// Negative values that are greater than `deadzone_lowerbound` will be rounded up to 0.0.
    deadzone_lowerbound: f32,
    /// Values that are lower than `livezone_lowerbound` will be rounded down to -1.0.
    livezone_lowerbound: f32,
    /// `threshold` defines the minimum difference between old and new values to apply the changes.
    threshold: f32,
}


/// Settings for all 3ds inputs.
///
/// ## Usage
///
/// It is used to create a `bevy` resource that stores the settings of every
/// [`_3dsAxis`]. If no user defined [`AxisSettings`]
/// are defined, the default settings of each are used as a fallback accordingly.
///
/// ## Note
///
/// The [`_3dsInputSettings`] are used to determine when raw 3ds events
/// should register as a [`_3dsEvent`]. Events that don't meet the change thresholds defined in [`_3dsInputSettings`]
/// will not register. To modify these settings, mutate the corresponding resource.
#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Debug, Default)]
pub struct _3dsInputSettings {
    /// The default axis settings.
    pub default_axis_settings: AxisSettings,
    /// The user defined axis settings.
    pub axis_settings: HashMap<_3dsAxis, AxisSettings>,
}

impl _3dsInputSettings {

    /// Returns the [`AxisSettings`] of the `axis`.
    ///
    /// If no user defined [`AxisSettings`] are specified the default [`AxisSettings`] get returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_3ds::input::_3ds::{_3dsInputSettings, _3dsAxis, _3dsAxisType};
    /// #
    /// # let settings = _3dsInputSettings::default();
    /// let axis = _3dsAxis::new(_3dsAxisType::LeftStickX);
    /// let axis_settings = settings.get_axis_settings(axis);
    /// ```
    pub fn get_axis_settings(&self, axis: _3dsAxis) -> &AxisSettings {
        self.axis_settings
            .get(&axis)
            .unwrap_or(&self.default_axis_settings)
    }
}
