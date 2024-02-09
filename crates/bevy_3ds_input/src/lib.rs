use bevy::app::Plugin;
use bevy::input::InputSystem;
use event::{axis_3ds_event_system, button_3ds_event_system, event_system_3ds, Axis3dsChangedEvent, Button3dsChangedEvent, CtruButtonChangedEvent, Event3ds};
use axis::{Axis3ds, Axis3dsType};
use button::{Button3ds, Button3dsType};
use bevy::input::{ButtonState, Input, Axis};
use bevy::app::PreUpdate;
use bevy::prelude::IntoSystemConfigs;
use ctru::services::hid::Hid;
use bevy::ecs::event::EventWriter;
use num_traits::pow::Pow;

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
                    event_system_3ds
                        .after(ctru_event_system),
                    button_3ds_event_system
                        .after(event_system_3ds),
                    axis_3ds_event_system
                        .after(event_system_3ds),
                )
                    .in_set(InputSystem),
            );

        app.register_type::<Button3dsType>()
            .register_type::<Button3ds>()
            .register_type::<Axis3dsType>()
            .register_type::<Axis3ds>();
    }
}

const DEADZONE_BOUND: f32 = 10.0;
const LIVEZONE_BOUND: f32 = 150.0;
pub fn ctru_event_system(
    mut events: EventWriter<Event3ds>,
) {
    // TODO: check if it is better to store a handle to the hid as a resource
    let mut hid = Hid::new().unwrap();
    hid.scan_input();
    for key in hid.keys_down() {
       if let Ok(button_type) = Button3dsType::try_from(key) {
            events.send(
                CtruButtonChangedEvent::new(button_type, ButtonState::Pressed).into(),
            );
        }
    }

    for key in hid.keys_up() {
       if let Ok(button_type) = Button3dsType::try_from(key) {
            events.send(
                CtruButtonChangedEvent::new(button_type, ButtonState::Released).into(),
            );
        }
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
    events.send(Axis3dsChangedEvent::new(Axis3dsType::CPadX, cpad_x / adjusted_livezone_bound).into());
    events.send(Axis3dsChangedEvent::new(Axis3dsType::CPadY, cpad_y / adjusted_livezone_bound).into());

    let volume: f32 = hid.volume_slider();
    if volume > 0.0 {
        events.send(Axis3dsChangedEvent::new(Axis3dsType::Volume, volume).into());
    }
    // TODO: add cstick (I don't think ctru-rs supports this)
    // TODO: add 3d slider axis
}
