use bevy::{ecs::system::ResMut, ui::ExtractedUiNodes};

struct UiNodeMeta {}

fn prepare_uinodes(mut nodes: ResMut<ExtractedUiNodes>) {
    nodes.uinodes.clear();
}
