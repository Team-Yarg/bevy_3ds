pub struct VertexAttrs(citro3d_sys::C3D_AttrInfo);

impl VertexAttrs {
    pub fn permutation(&self) -> u64 {
        self.0.permutation
    }
    pub fn count(&self) -> i32 {
        self.0.attrCount
    }
}

pub type ShaderLib = citro3d::shader::Library;

pub struct ShaderModule {
    pub lib: ShaderLib,
}

pub struct VertexState<'s> {
    pub shader: &'s ShaderLib,
    pub entry_point: usize,
}

pub struct RenderPipelineDescriptor<'s> {
    pub label: Option<&'static str>,
    pub vertex: VertexState<'s>,
}
