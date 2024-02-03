use bevy::{app::{Plugin, Update}, ecs::system::Res, input::{Axis, Input}};
use tracing::debug;
use crate::{axis::{Axis3ds, Axis3dsType}, button::Button3ds};

/// This plugin logs every keypress
pub struct Input3dsTestPlugin;
impl Plugin for Input3dsTestPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, button_text_system);
    }
}


fn button_text_system(
    buttons: Res<Input<Button3ds>>,
    axis: Res<Axis<Axis3ds>>,
) {
    let pressed_buttons = buttons.get_pressed();
    if let Some(volume) = axis.get(Axis3ds::new(Axis3dsType::Volume)) {
        if volume > 0.0 {
            debug!("volume: {}", volume);
        }
    }
    for button in pressed_buttons {
        let button_name = button.button_type.to_string();
        if button_name.contains("CPAD") {
            let x = axis.get(Axis3ds::new(Axis3dsType::CPadX)).unwrap();
            let y = axis.get(Axis3ds::new(Axis3dsType::CPadY)).unwrap();
            debug!("x: {}, y: {}", x, y);
        }

        else {
            debug!("{}", button_name);
        }
    }
}
