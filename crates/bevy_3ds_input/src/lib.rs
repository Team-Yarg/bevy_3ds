use bevy::app::Plugin;
use bevy::input::InputSystem;
use event::{_3ds_axis_event_system, _3ds_button_event_system, _3ds_event_system, _3dsAxisChangedEvent, _3dsButtonChangedEvent, _3dsEvent};
use settings::{_3dsInputSettings, _3dsAxisSettings};
use axis::{_3dsAxis, _3dsAxisType};
use button::{_3dsButton, _3dsButtonType};
use bevy::input::{ButtonState, Input, Axis};
use bevy::app::PreUpdate;
use bevy::prelude::IntoSystemConfigs;
use ctru::services::hid::{Hid, KeyPad};
use bevy::ecs::event::EventWriter;

pub mod axis;
pub mod button;
pub mod settings;
pub mod event;

pub struct InputPlugin;
impl Plugin for InputPlugin {
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
                    ctru_event_system,
                    _3ds_event_system
                        .after(ctru_event_system),
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

pub fn ctru_event_system(
    mut events: EventWriter<_3dsEvent>,
) {
    let mut hid = Hid::new().unwrap();
    hid.scan_input();
    for key in hid.keys_down() {
       let button_type: _3dsButtonType = ctru_to_bevy_keypad(key);
        events.send(
            _3dsButtonChangedEvent::new(button_type, ButtonState::Pressed).into(),
        );
    }

    for key in hid.keys_up() {
       let button_type: _3dsButtonType = ctru_to_bevy_keypad(key);
        events.send(
            _3dsButtonChangedEvent::new(button_type, ButtonState::Released).into(),
        );
    }
    // TODO: add axis
}

fn ctru_to_bevy_keypad(key: KeyPad) -> _3dsButtonType {
    match key {
        KeyPad::DPAD_RIGHT => {
            return _3dsButtonType::DPadRight;
        }

        KeyPad::DPAD_LEFT => {
            return _3dsButtonType::DPadLeft;
        }

        KeyPad::DPAD_UP => {
            return _3dsButtonType::DPadUp;
        }

        KeyPad::DPAD_DOWN => {
            return _3dsButtonType::DPadDown;
        }
        _ => {
            panic!("ctru key does not match");
        }
        // TODO: add more or just use KeyPad as enum in button.rs
    }

}
