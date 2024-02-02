use bevy::app::Plugin;
use bevy::input::InputSystem;
use event::{_3ds_axis_event_system, _3ds_button_event_system, _3ds_event_system, _3dsAxisChangedEvent, _3dsButtonChangedEvent, CtruButtonChangedEvent, _3dsEvent};
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
pub mod test;

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<_3dsButtonChangedEvent>()
            .add_event::<CtruButtonChangedEvent>()
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
    // TODO: check if it is better to store a handle to the hid as a resource
    let mut hid = Hid::new().unwrap();
    hid.scan_input();
    for key in hid.keys_down() {
       let button_type: _3dsButtonType = ctru_to_bevy_keypad(key);
        events.send(
            CtruButtonChangedEvent::new(button_type, ButtonState::Pressed).into(),
        );
    }

    for key in hid.keys_up() {
       let button_type: _3dsButtonType = ctru_to_bevy_keypad(key);
        events.send(
            CtruButtonChangedEvent::new(button_type, ButtonState::Released).into(),
        );
    }
    //TODO convert cpad_x & cpad_y to be between -1.0 and 1.0
    let (cpad_x, cpad_y) = hid.circlepad_position();
    events.send(_3dsAxisChangedEvent::new(_3dsAxisType::CPADX, cpad_x as f32).into());
    events.send(_3dsAxisChangedEvent::new(_3dsAxisType::CPADY, cpad_y as f32).into());
    // TODO: add cstick (I don't think ctru-rs supports this)
    // TODO: add volume slider axis
    // TODO: add 3d slider axis
}

fn ctru_to_bevy_keypad(key: KeyPad) -> _3dsButtonType {
    match key {
        KeyPad::B => {
            return _3dsButtonType::B;
        }

        KeyPad::A => {
            return _3dsButtonType::A;
        }

        KeyPad::Y => {
            return _3dsButtonType::Y;
        }

        KeyPad::X => {
            return _3dsButtonType::X;
        }

        KeyPad::SELECT => {
            return _3dsButtonType::SELECT;
        }

        KeyPad::START => {
            return _3dsButtonType::START;
        }

        KeyPad::DPAD_RIGHT => {
            return _3dsButtonType::DPAD_RIGHT;
        }

        KeyPad::DPAD_LEFT => {
            return _3dsButtonType::DPAD_LEFT;
        }

        KeyPad::DPAD_UP => {
            return _3dsButtonType::DPAD_UP;
        }

        KeyPad::DPAD_DOWN => {
            return _3dsButtonType::DPAD_DOWN;
        }

        KeyPad::ZL => {
            return _3dsButtonType::ZL;
        }

        KeyPad::ZR => {
            return _3dsButtonType::ZR;
        }

        KeyPad::L => {
            return _3dsButtonType::L;
        }

        KeyPad::R => {
            return _3dsButtonType::R;
        }

        _ => {
            return _3dsButtonType::NULL;
        }
    }
}
