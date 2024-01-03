use std::sync::RwLock;

use bevy::{
    app::App,
    ecs::{
        query::QueryState,
        system::{ReadOnlySystemParam, Resource, SystemState},
        world::World,
    },
};

use super::pass::{RenderCommand, RenderError, RenderPass};

struct RenderCommandState<C: RenderCommand> {
    state: SystemState<C::Param>,
}

impl<C: RenderCommand> RenderCommandState<C> {
    fn new(world: &mut World) -> Self {
        Self {
            state: SystemState::new(world),
        }
    }
}
impl<C: RenderCommand> Draw for RenderCommandState<C>
where
    C::Param: ReadOnlySystemParam,
{
    fn draw<'w, 'f>(
        &mut self,
        world: &'w World,
        pass: &'f mut RenderPass,
    ) -> Result<(), RenderError> {
        let param = self.state.get_manual(world);
        C::render(param, pass)
    }

    fn prepare(&mut self, world: &'_ World) {
        self.state.update_archetypes(world);
    }
}

/// See bevy render Draw for details, this is more or less a copy with our needed tweaks
pub trait Draw {
    fn prepare(&mut self, world: &'_ World) {}
    fn draw<'w, 'f>(
        &mut self,
        world: &'w World,
        pass: &'f mut RenderPass,
    ) -> Result<(), RenderError>;
}

#[derive(Default)]
struct DrawCommandsInner {
    commands: Vec<Box<dyn Draw + Send + Sync>>,
}

#[derive(Resource, Default)]
pub struct DrawCommands {
    inner: RwLock<DrawCommandsInner>,
}
impl DrawCommands {
    pub fn prepare(&self, world: &World) {
        let mut cmds = self.inner.write().unwrap();
        for act in cmds.commands.iter_mut() {
            act.prepare(world);
        }
    }
    pub fn run(&self, world: &World, pass: &mut RenderPass) -> Result<(), RenderError> {
        let mut cmds = self.inner.write().unwrap();
        for act in cmds.commands.iter_mut() {
            act.draw(world, pass)?;
        }
        Ok(())
    }
}

pub trait AppDrawCommandsExtra {
    fn add_render_command<C: RenderCommand + 'static>(&mut self) -> &mut Self
    where
        C::Param: ReadOnlySystemParam;
}

impl AppDrawCommandsExtra for App {
    fn add_render_command<C: RenderCommand + 'static>(&mut self) -> &mut Self
    where
        C::Param: ReadOnlySystemParam,
    {
        let cmd = Box::new(RenderCommandState::<C>::new(&mut self.world));
        let cmds = self.world.resource::<DrawCommands>();
        cmds.inner.write().unwrap().commands.push(cmd);
        self
    }
}
