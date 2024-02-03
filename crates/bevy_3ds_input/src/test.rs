use bevy::{app::{Plugin, Update}, ecs::system::Res, input::{Axis, Input}};
use tracing::debug;
use crate::{axis::{_3dsAxis, _3dsAxisType}, button::_3dsButton};

/// This plugin logs every keypress
pub struct _3dsInputTestPlugin;
impl Plugin for _3dsInputTestPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, button_text_system);
    }
}


fn button_text_system(
    buttons: Res<Input<_3dsButton>>,
    axis: Res<Axis<_3dsAxis>>,
) {
    let pressed_buttons = buttons.get_pressed();
    if let Some(volume) = axis.get(_3dsAxis::new(_3dsAxisType::Volume)) {
        if volume > 0.0 {
            debug!("volume: {}", volume);
        }
    }
    for button in pressed_buttons {
        let button_name = button.button_type.to_string();
        if button_name.contains("CPAD") {
            let x = axis.get(_3dsAxis::new(_3dsAxisType::CPadX)).unwrap();
            let y = axis.get(_3dsAxis::new(_3dsAxisType::CPadY)).unwrap();
            debug!("x: {}, y: {}", x, y);
        }

        else {
            debug!("{}", button_name);
        }
    }
}
