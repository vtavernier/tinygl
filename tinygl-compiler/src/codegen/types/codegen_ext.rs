use quote::{format_ident, quote};

use crate::types::{AtomType, GenericType, ItemOrArrayType, MatrixType, VectorType};

// TODO: Use a formatter
// TODO: bvec mapping is broken
// TODO: dvec mapping is broken

pub struct ExtraArg {
    pub name: &'static str,
    pub ty: &'static str,
    pub val: &'static str,
}

pub trait CodegenExt {
    fn glsl_base_type(&self) -> &'static str;
    fn glsl_vec_name(&self) -> String;
    fn glsl_mat_name(&self) -> String;
    fn rust_value_type(&self) -> String;
    fn rust_primitive_type(&self) -> &'static str;
    fn uniform_method_name(&self) -> String;
    fn uniform_method_extra_args(&self) -> &[ExtraArg];
    fn uniform_count_arg(&self) -> Option<usize>;

    fn uniform_method_extra_args_with_ty(&self) -> Option<proc_macro2::TokenStream> {
        let args = self.uniform_method_extra_args();

        if args.is_empty() {
            return None;
        }

        let name = args.iter().map(|e| format_ident!("{}", e.name));
        let ty = args
            .iter()
            .map(|e| -> syn::Type { syn::parse_str(e.ty).unwrap() });

        Some(quote! {
            #(#name: #ty),*
        })
    }

    fn uniform_method_extra_args_no_ty(&self) -> Option<proc_macro2::TokenStream> {
        let args = self.uniform_method_extra_args();

        if args.is_empty() {
            return None;
        }

        let name = args.iter().map(|e| format_ident!("{}", e.name));

        Some(quote! {
            #(#name),*
        })
    }
}

impl CodegenExt for AtomType {
    fn glsl_base_type(&self) -> &'static str {
        AtomType::glsl_base_type(self)
    }

    fn glsl_vec_name(&self) -> String {
        AtomType::glsl_vec_name(self)
    }

    fn glsl_mat_name(&self) -> String {
        AtomType::glsl_mat_name(self)
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
        format!(
            "1{}",
            match self {
                Self::Int => "i",
                Self::Float => "f",
                Self::Double => "d",
                Self::UInt => "ui",
                // TODO: Check how to handle bool
                Self::Bool => "i",
            }
        )
    }

    fn uniform_method_extra_args(&self) -> &[ExtraArg] {
        &[]
    }

    fn uniform_count_arg(&self) -> Option<usize> {
        None
    }
}

impl CodegenExt for VectorType {
    fn glsl_base_type(&self) -> &'static str {
        self.base_type.glsl_base_type()
    }

    fn glsl_vec_name(&self) -> String {
        VectorType::glsl_vec_name(self)
    }

    fn glsl_mat_name(&self) -> String {
        panic!("cannot format a vector as a matrix")
    }

    fn rust_value_type(&self) -> String {
        format!(
            "impl ::std::convert::AsRef<[{}; {}]>",
            self.base_type.rust_value_type(),
            self.components,
        )
    }

    fn rust_primitive_type(&self) -> &'static str {
        self.base_type.rust_primitive_type()
    }

    fn uniform_method_name(&self) -> String {
        format!(
            "{}{}v",
            self.components,
            match self.base_type {
                AtomType::Int => "i",
                AtomType::Float => "f",
                AtomType::Double => "d",
                AtomType::UInt => "ui",
                // TODO: Check how to handle bool
                AtomType::Bool => "iv",
            }
        )
    }

    fn uniform_method_extra_args(&self) -> &[ExtraArg] {
        &[]
    }

    fn uniform_count_arg(&self) -> Option<usize> {
        Some(1)
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
        MatrixType::glsl_mat_name(self)
    }

    fn rust_value_type(&self) -> String {
        format!(
            "impl ::std::convert::AsRef<[[{ty}; {n}]; {n}]>",
            ty = self.base_type.rust_value_type(),
            n = self.n,
        )
    }

    fn rust_primitive_type(&self) -> &'static str {
        self.base_type.rust_primitive_type()
    }

    fn uniform_method_name(&self) -> String {
        format!(
            "_matrix{}{}v",
            self.n,
            match self.base_type {
                AtomType::Float => "f",
                AtomType::Double => "d",
                // No matrices of other types
                _ => "",
            }
        )
    }

    fn uniform_method_extra_args(&self) -> &[ExtraArg] {
        &[ExtraArg {
            name: "transpose",
            ty: "bool",
            val: "transpose as u8",
        }]
    }

    fn uniform_count_arg(&self) -> Option<usize> {
        Some(1)
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

    fn uniform_method_extra_args(&self) -> &[ExtraArg] {
        match self {
            Self::Atom(atom) => atom.uniform_method_extra_args(),
            Self::Vector(vector) => vector.uniform_method_extra_args(),
            Self::Matrix(matrix) => matrix.uniform_method_extra_args(),
        }
    }

    fn uniform_count_arg(&self) -> Option<usize> {
        match self {
            Self::Atom(atom) => atom.uniform_count_arg(),
            Self::Vector(vector) => vector.uniform_count_arg(),
            Self::Matrix(matrix) => matrix.uniform_count_arg(),
        }
    }
}

impl CodegenExt for ItemOrArrayType {
    fn glsl_base_type(&self) -> &'static str {
        match self {
            Self::Item(item) => item.glsl_base_type(),
            Self::Array(item, _size) => item.glsl_base_type(),
            Self::Image { .. } => "image",
        }
    }

    fn glsl_vec_name(&self) -> String {
        match self {
            Self::Item(item) => item.glsl_vec_name(),
            Self::Array(item, _size) => item.glsl_vec_name(),
            Self::Image { .. } => "image".to_owned(),
        }
    }

    fn glsl_mat_name(&self) -> String {
        match self {
            Self::Item(item) => item.glsl_mat_name(),
            Self::Array(item, _size) => item.glsl_mat_name(),
            Self::Image { .. } => "image".to_owned(),
        }
    }

    fn rust_value_type(&self) -> String {
        match self {
            Self::Item(item) => item.rust_value_type(),
            Self::Array(item, size) => format!("&[{}; {}]", item.glsl_mat_name(), size),
            Self::Image { .. } => GenericType::Atom(AtomType::UInt).rust_value_type(),
        }
    }

    fn rust_primitive_type(&self) -> &'static str {
        match self {
            Self::Item(item) => item.rust_primitive_type(),
            Self::Array(item, _size) => item.rust_primitive_type(),
            Self::Image { .. } => GenericType::Atom(AtomType::UInt).rust_primitive_type(),
        }
    }

    fn uniform_method_name(&self) -> String {
        match self {
            Self::Item(item) => item.uniform_method_name(),
            Self::Array(item, _size) => item.uniform_method_name(),
            Self::Image { .. } => GenericType::Atom(AtomType::UInt).uniform_method_name(),
        }
    }

    fn uniform_method_extra_args(&self) -> &[ExtraArg] {
        match self {
            Self::Item(item) => item.uniform_method_extra_args(),
            Self::Array(item, _size) => item.uniform_method_extra_args(),
            Self::Image { .. } => GenericType::Atom(AtomType::UInt).uniform_method_extra_args(),
        }
    }

    fn uniform_count_arg(&self) -> Option<usize> {
        match self {
            Self::Item(item) => item.uniform_count_arg(),
            Self::Array(_, size) => Some(*size as usize),
            Self::Image { .. } => GenericType::Atom(AtomType::UInt).uniform_count_arg(),
        }
    }
}
