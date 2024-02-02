use bevy::{app::{Plugin, Update}, ecs::system::Res, input::Input};
use tracing::debug;
use std::any::type_name_of_val;
use crate::button::{_3dsButton, _3dsButtonType};

/// This plugin logs every keypress
pub struct _3dsInputTestPlugin;
impl Plugin for _3dsInputTestPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, button_text_system);
    }
}


fn button_text_system(
    buttons: Res<Input<_3dsButton>>,
) {
    let pressed_buttons = buttons.get_pressed();
    for button in pressed_buttons {
        let button_name = button.button_type.to_string();
        debug!("{}", button_name);
    }
}
