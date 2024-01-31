use bevy::app::Plugin;
pub struct Bevy3dsInputPlugin;
pub mod axis;
pub mod button;
pub mod settings;
impl Plugin for Bevy3dsInputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        todo!()
    }
}
