use super::*;

// TODO: Use a formatter
// TODO: bvec mapping is broken
// TODO: dvec mapping is broken

pub trait CodegenExt {
    fn glsl_base_type(&self) -> &'static str;
    fn glsl_vec_name(&self) -> String;
    fn glsl_mat_name(&self) -> String;
    fn rust_value_type(&self) -> String;
    fn rust_primitive_type(&self) -> &'static str;
    fn uniform_method_name(&self) -> String;
}

impl CodegenExt for AtomType {
    fn glsl_base_type(&self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Float => "float",
            Self::Double => "double",
            Self::UInt => "uint",
            Self::Bool => "bool",
        }
    }

    fn glsl_vec_name(&self) -> String {
        match self {
            Self::Int => "ivec",
            Self::Float => "vec",
            Self::Double => "dvec",
            Self::UInt => "uvec",
            Self::Bool => "bvec",
        }
        .into()
    }

    fn glsl_mat_name(&self) -> String {
        match self {
            Self::Float => "mat",
            Self::Double => "dmat",
            _ => panic!("cannot use mat_name on non-float"),
        }
        .into()
    }

    fn rust_value_type(&self) -> String {
        self.rust_primitive_type().into()
    }

    fn rust_primitive_type(&self) -> &'static str {
        match self {
            Self::Int => "i32",
            Self::Float => "f32",
            Self::Double => "f64",
            Self::UInt => "u32",
            Self::Bool => "bool",
        }
    }

    fn uniform_method_name(&self) -> String {
        format!("1_{}", self.rust_primitive_type())
    }
}

impl CodegenExt for VectorType {
    fn glsl_base_type(&self) -> &'static str {
        self.base_type.glsl_base_type()
    }

    fn glsl_vec_name(&self) -> String {
        format!("{}{}", self.base_type.glsl_vec_name(), self.components)
    }

    fn glsl_mat_name(&self) -> String {
        panic!("cannot format a vector as a matrix")
    }

    fn rust_value_type(&self) -> String {
        format!(
            "::tinygl::cgmath::Vector{}<{}>",
            self.components,
            self.base_type.rust_value_type()
        )
    }

    fn rust_primitive_type(&self) -> &'static str {
        self.base_type.rust_primitive_type()
    }

    fn uniform_method_name(&self) -> String {
        format!(
            "{}_{}",
            self.components,
            self.base_type.rust_primitive_type()
        )
    }
}

impl CodegenExt for MatrixType {
    fn glsl_base_type(&self) -> &'static str {
        self.base_type.glsl_base_type()
    }

    fn glsl_vec_name(&self) -> String {
        panic!("cannot format a matrix as a vector")
    }

    fn glsl_mat_name(&self) -> String {
        format!("{}{}", self.base_type.glsl_mat_name(), self.n)
    }

    fn rust_value_type(&self) -> String {
        format!(
            "::tinygl::cgmath::Matrix{}<{}>",
            self.n,
            self.base_type.rust_value_type()
        )
    }

    fn rust_primitive_type(&self) -> &'static str {
        self.base_type.rust_primitive_type()
    }

    fn uniform_method_name(&self) -> String {
        format!("matrix_{}_{}", self.n, self.rust_primitive_type())
    }
}

impl CodegenExt for GenericType {
    fn glsl_base_type(&self) -> &'static str {
        match self {
            Self::Atom(atom) => atom.glsl_base_type(),
            Self::Vector(vector) => vector.glsl_base_type(),
            Self::Matrix(matrix) => matrix.glsl_base_type(),
        }
    }

    fn glsl_vec_name(&self) -> String {
        match self {
            Self::Atom(atom) => atom.glsl_vec_name(),
            Self::Vector(vector) => vector.glsl_vec_name(),
            Self::Matrix(matrix) => matrix.glsl_vec_name(),
        }
    }

    fn glsl_mat_name(&self) -> String {
        match self {
            Self::Atom(atom) => atom.glsl_mat_name(),
            Self::Vector(vector) => vector.glsl_mat_name(),
            Self::Matrix(matrix) => matrix.glsl_mat_name(),
        }
    }

    fn rust_value_type(&self) -> String {
        match self {
            Self::Atom(atom) => atom.rust_value_type(),
            Self::Vector(vector) => vector.rust_value_type(),
            Self::Matrix(matrix) => matrix.rust_value_type(),
        }
    }

    fn rust_primitive_type(&self) -> &'static str {
        match self {
            Self::Atom(atom) => atom.rust_primitive_type(),
            Self::Vector(vector) => vector.rust_primitive_type(),
            Self::Matrix(matrix) => matrix.rust_primitive_type(),
        }
    }

    fn uniform_method_name(&self) -> String {
        match self {
            Self::Atom(atom) => atom.uniform_method_name(),
            Self::Vector(vector) => vector.uniform_method_name(),
            Self::Matrix(matrix) => matrix.uniform_method_name(),
        }
    }
}

impl CodegenExt for ItemOrArrayType {
    fn glsl_base_type(&self) -> &'static str {
        match self {
            Self::Item(item) => item.glsl_base_type(),
            Self::Array(item, _size) => item.glsl_base_type(),
        }
    }

    fn glsl_vec_name(&self) -> String {
        match self {
            Self::Item(item) => item.glsl_vec_name(),
            Self::Array(item, _size) => item.glsl_vec_name(),
        }
    }

    fn glsl_mat_name(&self) -> String {
        match self {
            Self::Item(item) => item.glsl_mat_name(),
            Self::Array(item, _size) => item.glsl_mat_name(),
        }
    }

    fn rust_value_type(&self) -> String {
        match self {
            Self::Item(item) => item.rust_value_type(),
            Self::Array(item, size) => format!("&[{}; {}]", item.glsl_mat_name(), size),
        }
    }

    fn rust_primitive_type(&self) -> &'static str {
        match self {
            Self::Item(item) => item.rust_primitive_type(),
            Self::Array(item, _size) => item.rust_primitive_type(),
        }
    }

    fn uniform_method_name(&self) -> String {
        match self {
            Self::Item(item) => item.uniform_method_name(),
            Self::Array(item, _size) => item.uniform_method_name(),
        }
    }
}
