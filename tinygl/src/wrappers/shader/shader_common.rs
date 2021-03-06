/// Common traits to binary and source shaders
pub trait ShaderCommon {
    fn kind(&self) -> u32;
    fn name(&self) -> crate::gl::ShaderName;
}
