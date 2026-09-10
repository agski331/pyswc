

macro_rules! ast_node_variant {
    ($base:ty, $py_name:ident, $swc_ty:ty, { $($field:ident : $field_ty:ty = $conv:expr),* $(,)? }) => {
        #[::pyo3::pyclass(extends = $base)]
        pub struct $py_name {
            $(#[pyo3(get)] pub $field: $field_ty,)*
        }

        impl $py_name {
            pub fn build(py: ::pyo3::Python<'_>, node: $swc_ty) -> ::pyo3::PyResult<Self> {
                Ok($py_name {
                    $( $field: $conv(py, node.$field)?, )*
                })
            }
        }
    };
}

pub(crate) use ast_node_variant;
