use axis::{Axis3ds, Axis3dsType};
use bevy::app::Plugin;
use bevy::app::PreUpdate;
use bevy::ecs::event::EventWriter;
use bevy::input::InputSystem;
use bevy::input::{Axis, ButtonState, Input};
use bevy::prelude::IntoSystemConfigs;
use button::{Button3ds, Button3dsType};
use ctru::services::hid::Hid;
use ctru::services::hid::KeyPad;
use event::{
    axis_3ds_event_system, button_3ds_event_system, event_system_3ds, Axis3dsChangedEvent,
    Button3dsChangedEvent, CtruButtonChangedEvent, Event3ds,
};
use num_traits::pow::Pow;

use std::mem::MaybeUninit;

pub mod axis;
pub mod button;
pub mod event;
pub mod test;

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<Button3dsChangedEvent>()
            .add_event::<CtruButtonChangedEvent>()
            .add_event::<Axis3dsChangedEvent>()
            .add_event::<Event3ds>()
            .init_resource::<Input<Button3ds>>()
            .init_resource::<Axis<Axis3ds>>()
            .add_systems(
                PreUpdate,
                (
                    ctru_event_system,
                    event_system_3ds.after(ctru_event_system),
                    button_3ds_event_system.after(event_system_3ds),
                    axis_3ds_event_system.after(event_system_3ds),
                )
                    .in_set(InputSystem),
            );

        app.register_type::<Button3dsType>()
            .register_type::<Button3ds>()
            .register_type::<Axis3dsType>()
            .register_type::<Axis3ds>();
    }
}

// this relies on irrst already being initialised, which is the case currently
// but cannot be depended on in general (it will probably crash on Old 3DS)
pub fn cstick_position() -> (i16, i16) {
    let res = unsafe {
        let mut res = MaybeUninit::uninit();
        ctru_sys::irrstCstickRead(res.as_mut_ptr());
        res.assume_init()
    };

    (res.dx, res.dy)
}

const DEADZONE_BOUND: f32 = 20.0;
const LIVEZONE_BOUND: f32 = 150.0;
pub fn ctru_event_system(mut events: EventWriter<Event3ds>) {
    // TODO: check if it is better to store a handle to the hid as a resource
    let mut hid = Hid::new().unwrap();
    hid.scan_input();
    for key in hid.keys_down() {
        if let Ok(button_type) = Button3dsType::try_from(key) {
            events.send(CtruButtonChangedEvent::new(button_type, ButtonState::Pressed).into());
        }
    }

    for key in hid.keys_up() {
        if let Ok(button_type) = Button3dsType::try_from(key) {
            events.send(CtruButtonChangedEvent::new(button_type, ButtonState::Released).into());
        }
    }
    let (cpad_x, cpad_y) = hid.circlepad_position();
    let mut cpad_x = cpad_x as f32;
    let mut cpad_y = cpad_y as f32;

    // calculate the distance from the origin
    let distance = (cpad_x * cpad_x + cpad_y * cpad_y).sqrt();

    if distance < DEADZONE_BOUND {
        cpad_x = 0.0;
        cpad_y = 0.0;
        events.send(
            CtruButtonChangedEvent::new(Button3dsType::CPadDown, ButtonState::Released).into(),
        );
        events
            .send(CtruButtonChangedEvent::new(Button3dsType::CPadUp, ButtonState::Released).into());
        events.send(
            CtruButtonChangedEvent::new(Button3dsType::CPadLeft, ButtonState::Released).into(),
        );
        events.send(
            CtruButtonChangedEvent::new(Button3dsType::CPadRight, ButtonState::Released).into(),
        );
    } else {
        if cpad_x < 0.0 {
            events.send(
                CtruButtonChangedEvent::new(Button3dsType::CPadRight, ButtonState::Released).into(),
            );
            events.send(
                CtruButtonChangedEvent::new(Button3dsType::CPadLeft, ButtonState::Pressed).into(),
            );
        } else {
            events.send(
                CtruButtonChangedEvent::new(Button3dsType::CPadLeft, ButtonState::Released).into(),
            );
            events.send(
                CtruButtonChangedEvent::new(Button3dsType::CPadRight, ButtonState::Pressed).into(),
            );
        }

        if cpad_y < 0.0 {
            events.send(
                CtruButtonChangedEvent::new(Button3dsType::CPadUp, ButtonState::Released).into(),
            );
            events.send(
                CtruButtonChangedEvent::new(Button3dsType::CPadDown, ButtonState::Pressed).into(),
            );
        } else {
            events.send(
                CtruButtonChangedEvent::new(Button3dsType::CPadDown, ButtonState::Released).into(),
            );
            events.send(
                CtruButtonChangedEvent::new(Button3dsType::CPadUp, ButtonState::Pressed).into(),
            );
        }

        cpad_x -= cpad_x * DEADZONE_BOUND / distance;
        cpad_y -= cpad_y * DEADZONE_BOUND / distance;
    }
    let adjusted_livezone_bound = LIVEZONE_BOUND - DEADZONE_BOUND; // so that scale is smooth
    events.send(
        Axis3dsChangedEvent::new(Axis3dsType::CPadX, cpad_x / adjusted_livezone_bound).into(),
    );
    events.send(
        Axis3dsChangedEvent::new(Axis3dsType::CPadY, cpad_y / adjusted_livezone_bound).into(),
    );

    let (cstick_x, cstick_y) = cstick_position();
    let mut cstick_x = cstick_x as f32;
    let mut cstick_y = cstick_y as f32;

    // calculate the distance from the origin
    let distance = (cstick_x * cstick_x + cstick_y * cstick_y).sqrt();

    if distance < DEADZONE_BOUND {
        cstick_x = 0.0;
        cstick_y = 0.0;
        events.send(
            CtruButtonChangedEvent::new(Button3dsType::CStickDown, ButtonState::Released).into(),
        );
        events.send(
            CtruButtonChangedEvent::new(Button3dsType::CStickUp, ButtonState::Released).into(),
        );
        events.send(
            CtruButtonChangedEvent::new(Button3dsType::CStickLeft, ButtonState::Released).into(),
        );
        events.send(
            CtruButtonChangedEvent::new(Button3dsType::CStickRight, ButtonState::Released).into(),
        );
    } else {
        if cstick_x < 0.0 {
            events.send(
                CtruButtonChangedEvent::new(Button3dsType::CStickRight, ButtonState::Released)
                    .into(),
            );
            events.send(
                CtruButtonChangedEvent::new(Button3dsType::CStickLeft, ButtonState::Pressed).into(),
            );
        } else {
            events.send(
                CtruButtonChangedEvent::new(Button3dsType::CStickLeft, ButtonState::Released)
                    .into(),
            );
            events.send(
                CtruButtonChangedEvent::new(Button3dsType::CStickRight, ButtonState::Pressed)
                    .into(),
            );
        }

        if cstick_y < 0.0 {
            events.send(
                CtruButtonChangedEvent::new(Button3dsType::CStickUp, ButtonState::Released).into(),
            );
            events.send(
                CtruButtonChangedEvent::new(Button3dsType::CStickDown, ButtonState::Pressed).into(),
            );
        } else {
            events.send(
                CtruButtonChangedEvent::new(Button3dsType::CStickDown, ButtonState::Released)
                    .into(),
            );
            events.send(
                CtruButtonChangedEvent::new(Button3dsType::CStickUp, ButtonState::Pressed).into(),
            );
        }

        cstick_x -= cstick_x * DEADZONE_BOUND / distance;
        cstick_y -= cstick_y * DEADZONE_BOUND / distance;
    }
    events.send(
        Axis3dsChangedEvent::new(Axis3dsType::CStickX, cstick_x / adjusted_livezone_bound).into(),
    );
    events.send(
        Axis3dsChangedEvent::new(Axis3dsType::CStickY, cstick_y / adjusted_livezone_bound).into(),
    );

    let volume: f32 = hid.volume_slider();
    if volume > 0.0 {
        events.send(Axis3dsChangedEvent::new(Axis3dsType::Volume, volume).into());
    }
    // TODO: add cstick (I don't think ctru-rs supports this)
    // TODO: add 3d slider axis
}
