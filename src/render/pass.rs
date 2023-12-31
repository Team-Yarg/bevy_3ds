use bevy::ecs::{
    query::ROQueryItem,
    system::{SystemParam, SystemParamItem},
};

pub struct RenderPass {}

#[derive(Debug)]
pub enum RenderError {
    Generic,
}
impl Error for RenderError {}
impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::Generic => f.write_str("generic"),
        }
    }
}

pub trait RenderCommand {
    type Param: SystemParam;
    type ItemData: ReadOnlyQueryData;

    fn render<'w>(
        entity: ROQueryItem<'w, Self::ItemData>,
        param: &SystemParamItem<'w, '_, Self::Param>,
        pass: &mut RenderPass,
    ) -> Result<(), RenderError>;
}
