use super::shader::PicaShader;

pub struct VertexAttribute {
    pub register: citro3d::attrib::Register,
    pub format: citro3d::attrib::Format,
    pub count: u8,
}

#[derive(Default, Debug)]
pub struct VertexAttrs(pub(super) citro3d::attrib::Info);

impl VertexAttrs {
    pub fn from_citro3d(info: citro3d::attrib::Info) -> Self {
        Self(info)
    }
    pub fn new(attrs: &[VertexAttribute]) -> citro3d::Result<Self> {
        let mut me = Self::default();
        for attr in attrs {
            me.add_attr(attr)?;
        }
        Ok(me)
    }
    pub fn add_attr(&mut self, attr: &VertexAttribute) -> citro3d::Result<()> {
        self.0.add_loader(attr.register, attr.format, attr.count)?;
        Ok(())
    }
    pub fn permutation(&self) -> u64 {
        self.0.permutation()
    }
    pub fn count(&self) -> i32 {
        self.0.attr_count()
    }
}

pub type ShaderLib = citro3d::shader::Library;

pub struct ShaderModule {
    pub lib: ShaderLib,
}

pub struct VertexState<'f> {
    pub shader: &'f PicaShader,
    pub entry_point: usize,
    pub attrs: VertexAttrs,
}

pub struct RenderPipelineDescriptor<'s> {
    pub label: Option<&'static str>,
    pub vertex: VertexState<'s>,
}
