use bevy::reflect::Reflect;
use bevy::ecs::system::Resource;
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
    /// # use bevy_3ds::input::settings::{_3dsInputSettings};
    /// # use bevy_3ds::input::axis::{_3dsAxis, _3dsAxisType};
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
