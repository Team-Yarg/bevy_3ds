use bevy::app::Plugin;
use bevy::input::InputSystem;
use event::{_3ds_axis_event_system, _3ds_button_event_system, _3ds_event_system, _3dsAxisChangedEvent, _3dsButtonChangedEvent, _3dsEvent};
use settings::{_3dsInputSettings, _3dsAxisSettings};
use axis::{_3dsAxis, _3dsAxisType};
use button::{_3dsButton, _3dsButtonType};
use bevy::input::{Input, Axis};
use bevy::app::PreUpdate;
use bevy::prelude::IntoSystemConfigs;

pub mod axis;
pub mod button;
pub mod settings;
pub mod event;

pub struct Bevy3dsInputPlugin;
impl Plugin for Bevy3dsInputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<_3dsButtonChangedEvent>()
            .add_event::<_3dsAxisChangedEvent>()
            .add_event::<_3dsEvent>()
            .init_resource::<_3dsInputSettings>()
            .init_resource::<Input<_3dsButton>>()
            .init_resource::<Axis<_3dsAxis>>()
            .add_systems(
                PreUpdate,
                (
                    _3ds_event_system,
                    _3ds_button_event_system
                        .after(_3ds_event_system),
                    _3ds_axis_event_system
                        .after(_3ds_event_system),
                )
                    .in_set(InputSystem),
            );

        app.register_type::<_3dsButtonType>()
            .register_type::<_3dsButton>()
            .register_type::<_3dsAxisType>()
            .register_type::<_3dsAxis>()
            .register_type::<_3dsInputSettings>()
            .register_type::<_3dsAxisSettings>();
    }
}
