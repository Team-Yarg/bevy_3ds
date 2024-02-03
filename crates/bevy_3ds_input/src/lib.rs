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
use num_traits::pow::Pow;

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

const DEADZONE_BOUND: f32 = 10.0;
const LIVEZONE_BOUND: f32 = 150.0;
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
    let (cpad_x, cpad_y) = hid.circlepad_position();
    let mut cpad_x: f32 = cpad_x as f32;
    let mut cpad_y: f32 = cpad_y as f32;
    // calculate the distance from the origin
    let distance: f32 = ((cpad_x.pow(2) + cpad_y.pow(2)) as f32).sqrt();

    if distance < DEADZONE_BOUND {
        cpad_x = 0.0;
        cpad_y = 0.0;
    } else {
        if cpad_x < 0.0 {
            cpad_x += DEADZONE_BOUND;
        } else {
            cpad_x -= DEADZONE_BOUND;
        }

        if cpad_y < 0.0 {
            cpad_y += DEADZONE_BOUND;
        } else {
            cpad_y -= DEADZONE_BOUND;
        }
    }
    let adjusted_livezone_bound = LIVEZONE_BOUND - DEADZONE_BOUND; // so that scale is smooth
    if cpad_x > 0.0 {
        events.send(_3dsAxisChangedEvent::new(_3dsAxisType::CPadX, cpad_x / adjusted_livezone_bound).into());
    }
    if cpad_y > 0.0 {
        events.send(_3dsAxisChangedEvent::new(_3dsAxisType::CPadY, cpad_y / adjusted_livezone_bound).into());
    }

    let volume: f32 = hid.volume_slider();
    if volume > 0.0 {
        events.send(_3dsAxisChangedEvent::new(_3dsAxisType::Volume, volume).into());
    }
    // TODO: add cstick (I don't think ctru-rs supports this)
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
            return _3dsButtonType::Select;
        }

        KeyPad::START => {
            return _3dsButtonType::Start;
        }

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

        KeyPad::CPAD_RIGHT => {
            return _3dsButtonType::CPadRight;
        }

        KeyPad::CPAD_LEFT => {
            return _3dsButtonType::CPadLeft;
        }

        KeyPad::CPAD_UP => {
            return _3dsButtonType::CPadUp;
        }

        KeyPad::CPAD_DOWN => {
            return _3dsButtonType::CPadDown;
        }

        KeyPad::CSTICK_RIGHT => {
            return _3dsButtonType::CStickRight;
        }

        KeyPad::CSTICK_LEFT => {
            return _3dsButtonType::CStickLeft;
        }

        KeyPad::CSTICK_UP => {
            return _3dsButtonType::CStickUp;
        }

        KeyPad::CSTICK_DOWN => {
            return _3dsButtonType::CStickDown;
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
            return _3dsButtonType::Null;
        }
    }
}
