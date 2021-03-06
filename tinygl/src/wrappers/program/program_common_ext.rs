use crate::wrappers::{UniformBlock, UniformLocation};
use crate::Context;

use super::ProgramCommon;

pub trait ProgramCommonExt {
    unsafe fn use_program(&self, gl: &Context);
    fn get_uniform_location(&self, gl: &Context, name: &str) -> UniformLocation;
    fn get_uniform_block_index(&self, gl: &Context, name: &str) -> Option<UniformBlock>;
}

impl<T: ProgramCommon> ProgramCommonExt for T {
    unsafe fn use_program(&self, gl: &Context) {
        gl.use_program(Some(self.name()));
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn get_uniform_location(&self, gl: &Context, name: &str) -> UniformLocation {
        use std::ffi::CString;

        UniformLocation::new(self.name(), unsafe {
            let name = CString::new(name).expect("invalid location identifier");
            let loc = gl.get_uniform_location(self.name(), name.as_ptr());

            if loc < 0 {
                None
            } else {
                Some(loc)
            }
        })
    }

    #[cfg(target_arch = "wasm32")]
    fn get_uniform_location(&self, gl: &Context, name: &str) -> UniformLocation {
        UniformLocation::new(gl.get_uniform_location(self.name(), name))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn get_uniform_block_index(&self, gl: &Context, name: &str) -> Option<UniformBlock> {
        use std::ffi::CString;

        let name = CString::new(name).expect("invalid uniform block name");

        unsafe {
            let index = gl.get_uniform_block_index(self.name(), name.as_ptr());
            UniformBlock::new(gl, self.name(), index)
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn get_uniform_block_index(&self, gl: &Context, name: &str) -> Option<UniformBlock> {
        let index = gl.get_uniform_block_index(self.name(), name);
        UniformBlock::new(gl, self.name().clone(), index)
    }
}
