use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use heck::CamelCase;
use heck::SnakeCase;

use crate::{shader_kind::ShaderKindInfo, Error, Result};

#[derive(Debug)]
pub enum TargetType {
    Automatic,
    SpirV,
    Glsl(spirv_cross::glsl::Version),
}

impl Default for TargetType {
    fn default() -> Self {
        Self::Automatic
    }
}

#[derive(Default)]
pub struct CompilerBuilder {
    skip_cargo: bool,
    dest: Option<PathBuf>,
    skip_spirv: bool,
    output_type: TargetType,
}

impl CompilerBuilder {
    pub fn skip_cargo(self, skip_cargo: bool) -> Self {
        Self { skip_cargo, ..self }
    }

    pub fn dest(self, dest: impl Into<PathBuf>) -> Self {
        Self {
            dest: Some(dest.into()),
            ..self
        }
    }

    pub fn skip_spirv(self, skip_spirv: bool) -> Self {
        Self { skip_spirv, ..self }
    }

    pub fn output_type(self, output_type: TargetType) -> Self {
        Self {
            output_type,
            ..self
        }
    }

    pub fn build(mut self) -> Result<Compiler> {
        // Are we building for WASM?
        let is_wasm = std::env::var("TARGET")
            .map(|v| v.starts_with("wasm32"))
            .unwrap_or(false);

        // Default path to OUT_DIR
        if self.dest.is_none() {
            self.dest = std::env::var("OUT_DIR").map(PathBuf::from).ok();
        }

        // If building for WASM, force source usage unless a specific version is required
        let output_type = match self.output_type {
            TargetType::Automatic => {
                if is_wasm {
                    TargetType::Glsl(spirv_cross::glsl::Version::V3_00Es)
                } else {
                    if self.skip_spirv {
                        TargetType::Glsl(spirv_cross::glsl::Version::V4_60)
                    } else {
                        TargetType::SpirV
                    }
                }
            }
            TargetType::SpirV => {
                if is_wasm {
                    return Err(Error::InvalidTargetType(self.output_type));
                } else {
                    if self.skip_spirv {
                        return Err(Error::InvalidSkipSpirV);
                    } else {
                        TargetType::SpirV
                    }
                }
            }
            TargetType::Glsl(version) => {
                if is_wasm {
                    match version {
                        spirv_cross::glsl::Version::V3_00Es
                        | spirv_cross::glsl::Version::V1_00Es => TargetType::Glsl(version),
                        _ => {
                            return Err(Error::InvalidTargetType(self.output_type));
                        }
                    }
                } else {
                    TargetType::Glsl(version)
                }
            }
        };

        Ok(Compiler {
            compiler: shaderc::Compiler::new().unwrap(),
            skip_cargo: self.skip_cargo,
            shader_names: Vec::new(),
            dest: self.dest.expect(
                "dest was not specified for the compiler and the OUT_DIR variable was not defined",
            ),
            skip_spirv: self.skip_spirv,
            output_type,
        })
    }
}

pub struct Compiler {
    compiler: shaderc::Compiler,
    shader_names: Vec<String>,
    skip_cargo: bool,
    dest: PathBuf,
    skip_spirv: bool,
    output_type: TargetType,
}

impl Compiler {
    fn write_shader(
        &self,
        shader: &str,
        binary_result: &shaderc::CompilationArtifact,
    ) -> Result<String> {
        let shader_file_name = format!(
            "{}{}",
            shader,
            if let TargetType::SpirV = self.output_type {
                ".spv"
            } else {
                ""
            }
        );

        // Write binary to .spv/.glsl file
        let mut output = File::create(&self.dest.join(&shader_file_name))?;

        match self.output_type {
            TargetType::SpirV => {
                // Just write spv file
                output.write_all(binary_result.as_binary_u8())?;
            }
            TargetType::Glsl(version) => {
                if self.skip_spirv {
                    // We skipped SPIR-V generation so just fix invalid stuff for OpenGL ES targets
                    // WebGL is more sensitive to leftovers from includes and stuff
                    // TODO: This is an ugly hack, maybe forbid skip_spirv + ES 3.00?
                    for l in binary_result.as_text().lines() {
                        if l.starts_with("#extension GL_GOOGLE_include_directive") {
                            continue;
                        } else if l.starts_with("#line") {
                            writeln!(output, "//{}", l)?;
                        } else {
                            writeln!(output, "{}", l)?;
                        }
                    }
                } else {
                    // Use spirv_cross to write valid code
                    let module = spirv_cross::spirv::Module::from_words(binary_result.as_binary());
                    let mut ast =
                        spirv_cross::spirv::Ast::<spirv_cross::glsl::Target>::parse(&module)?;

                    // Target the right GLSL version
                    ast.set_compiler_options(&spirv_cross::glsl::CompilerOptions {
                        version,
                        ..Default::default()
                    })
                    .unwrap();

                    write!(output, "{}", ast.compile()?)?;
                }
            }
            _ => unreachable!(),
        }

        Ok(shader_file_name)
    }

    fn write_rust_wrapper(
        &self,
        base_name: &str,
        shader: &str,
        kind: ShaderKindInfo,
        shader_file_name: &str,
        uniforms: &Vec<crate::reflect::FoundUniform>,
    ) -> Result<String> {
        let rs_file_name = String::from(base_name) + ".rs";

        // Write Rust interface code
        let output_rs = File::create(&self.dest.join(&rs_file_name)).unwrap();
        let mut wr = BufWriter::new(output_rs);

        let struct_name = (String::from(base_name) + "_shader").to_camel_case();
        let uniform_trait_name = (String::from(base_name) + "_uniforms").to_camel_case();

        writeln!(wr, "/// {} Rust wrapper", shader)?;
        writeln!(wr, "pub struct {} {{}}", struct_name)?;

        // Write struct for holding uniform locations
        writeln!(wr, "#[derive(Default)]")?;
        writeln!(wr, "pub struct {} {{", uniform_trait_name)?;

        for uniform in uniforms {
            let uniform_location_name = (uniform.name.clone() + "_location").to_snake_case();

            writeln!(wr, "    {name}: Option<<::tinygl::glow::Context as ::tinygl::glow::HasContext>::UniformLocation>,",
                name = uniform_location_name)?;
        }
        writeln!(wr, "}}")?;

        writeln!(wr, "impl {} {{", uniform_trait_name)?;
        // Write constructor
        writeln!(
            wr,
            "    pub fn new({prefix}gl: &::tinygl::Context, {prefix}program: <::tinygl::glow::Context as ::tinygl::glow::HasContext>::Program) -> Self {{",
            prefix = if let TargetType::Glsl(_) = self.output_type {
                ""
            } else {
                "_"
            })?;
        writeln!(
            wr,
            "        Self {{")?;

        for uniform in uniforms {
            let uniform_location_name = (uniform.name.clone() + "_location").to_snake_case();

            if let TargetType::Glsl(_) = self.output_type {
                // Source shader: find uniform locations from variable names
                writeln!(wr, "            {name}: unsafe {{ gl.get_uniform_location(program, \"{uniform_name}\") }},",
                    name = uniform_location_name,
                    uniform_name = uniform.name)?;
            } else {
                // Binary shader: assume locations form reflection on SPIR-V
                writeln!(wr, "            {name}: Some({location}),",
                    name = uniform_location_name,
                    location = uniform.location)?;
            }
        }

        writeln!(
            wr,
            "        }}")?;
        writeln!(wr, "    }}")?;

        // Write setter methods
        for uniform in uniforms {
            let ty = uniform.ty.unwrap();
            let uniform_location_name = (uniform.name.clone() + "_location").to_snake_case();

            writeln!(
                wr,
                "    pub fn set_{uniform_sc_name}(&self, gl: &::tinygl::Context, value: {type_name}) {{",
                uniform_sc_name = uniform.name.to_snake_case(),
                type_name = ty.cgmath_name()
            )?;

            writeln!(wr, "        use ::tinygl::HasContext;")?;

            writeln!(wr, "        unsafe {{ gl.uniform_{components}_{rstype}_slice(self.{location}.as_ref(), {what}) }};",
                components = ty.components(),
                rstype = ty.rstype(),
                location = uniform_location_name,
                what = ty.glow_value("value"))?;

            writeln!(wr, "    }}")?;
        }
        writeln!(wr, "}}")?;

        // A wrapped shader implements ShaderCommon
        writeln!(wr, "impl ::tinygl::ShaderCommon for {} {{", struct_name)?;
        writeln!(wr, "    fn get_kind() -> u32 {{")?;
        writeln!(wr, "        ::tinygl::gl::{}", kind.constant_name)?;
        writeln!(wr, "    }}")?;
        writeln!(wr, "}}")?;

        // Implement the right shader trait for the given output type
        if let TargetType::Glsl(_) = self.output_type {
            writeln!(
                wr,
                "impl ::tinygl::SourceShader<'static> for {} {{",
                struct_name
            )?;
            writeln!(wr, "    fn get_source() -> &'static str {{")?;
            writeln!(wr, "        include_str!(\"{}\")", shader_file_name)?;
            writeln!(wr, "    }}")?;
            writeln!(wr, "}}")?;
        } else {
            writeln!(
                wr,
                "impl ::tinygl::BinaryShader<'static> for {} {{",
                struct_name
            )?;
            writeln!(wr, "    fn get_binary() -> &'static [u8] {{")?;
            writeln!(wr, "        include_bytes!(\"{}\")", shader_file_name)?;
            writeln!(wr, "    }}")?;
            writeln!(wr, "}}")?;
        }

        Ok(rs_file_name)
    }

    pub fn wrap_shader(&mut self, source_path: impl AsRef<Path>) -> Result<()> {
        // Get full path to shader
        let source_path = std::fs::canonicalize(source_path)?;

        // Shader name
        let shader = source_path
            .file_name()
            .expect("source shader is not a file")
            .to_string_lossy()
            .to_owned();

        if !self.skip_cargo {
            // Notify cargo to rerun if the source changes
            println!("cargo:rerun-if-changed={}", source_path.display());
        }

        // Read GLSL source
        let source = std::fs::read_to_string(&source_path).unwrap();

        // Match shader type
        let kind = ShaderKindInfo::from_path(&source_path)
            .expect("no file extension on path, cannot determine shader type");

        let (res, rs_file_name) = {
            // Set callback
            let mut options = shaderc::CompileOptions::new().unwrap();

            // Default to OpenGL targets
            options.set_target_env(shaderc::TargetEnv::OpenGL, 0);

            // Set include callback
            let skip_cargo = self.skip_cargo;
            options.set_include_callback(move |name, _include_type, source, _depth| {
                // TODO: Circular includes?
                // TODO: Better include resolver?
                match std::fs::canonicalize(Path::new(&source).parent().unwrap().join(name)) {
                    Ok(full_path) => {
                        if !skip_cargo {
                            // Notify cargo to rerun if included file changed
                            println!("cargo:rerun-if-changed={}", full_path.display());
                        }

                        match std::fs::read_to_string(&full_path) {
                            Ok(content) => Ok(shaderc::ResolvedInclude {
                                resolved_name: full_path.to_string_lossy().to_string(),
                                content,
                            }),
                            Err(error) => Err(error.to_string()),
                        }
                    }
                    Err(error) => Err(error.to_string()),
                }
            });

            let compiler_result = if self.skip_spirv {
                // Only assemble source if we're skipping SPIR-V
                self.compiler.preprocess(
                    &source,
                    &source_path.to_string_lossy(),
                    "main",
                    Some(&options),
                )
            } else {
                // Compile into SPIR-V
                self.compiler.compile_into_spirv(
                    &source,
                    kind.shaderc_kind,
                    &source_path.to_string_lossy(),
                    "main",
                    Some(&options),
                )
            };

            match compiler_result {
                Ok(binary_result) => {
                    // TODO: Show compilation warnings from binary_result

                    // Base name to identify this shader
                    let base_name = shader.replace(".", "_");

                    // Extract uniforms from SPIR-V representation
                    let uniforms = if !self.skip_spirv {
                        // Extract uniform data
                        let mut loader = rspirv::mr::Loader::new();
                        rspirv::binary::parse_words(binary_result.as_binary(), &mut loader)
                            .unwrap();
                        let module = loader.module();

                        crate::reflect::find_uniforms(&module)
                    } else {
                        // TODO: Run glsl parser on transpiled source to extract active uniforms?
                        Vec::new()
                    };

                    let shader_file_name = self.write_shader(&shader, &binary_result)?;
                    let rs_file_name = self.write_rust_wrapper(
                        &base_name,
                        &shader,
                        kind,
                        &shader_file_name,
                        &uniforms,
                    )?;

                    (Ok(()), Some(rs_file_name))
                }
                Err(shaderc::Error::CompilationError(num_errors, errors)) => {
                    if !self.skip_cargo {
                        eprintln!("{}", errors);
                    }

                    (
                        Err(Error::CompilationError(num_errors as usize, errors)),
                        None,
                    )
                }
                Err(error) => panic!(error.to_string()),
            }
        };

        if let Some(rs_file_name) = rs_file_name {
            // Add to list of files to include
            self.shader_names.push(rs_file_name);
        }

        res
    }

    pub fn write_root_include(&self) -> Result<()> {
        // Write master shaders.rs file
        let output_rs = File::create(&self.dest.join("shaders.rs"))?;
        let mut wr = BufWriter::new(output_rs);

        for shader in &self.shader_names {
            writeln!(wr, "include!(\"{}\");", shader)?;
        }

        Ok(())
    }
}
