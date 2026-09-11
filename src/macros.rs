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

macro_rules! lazy_leaf_node {
    ($py_name:ident, $swc_ty:ty, { $($field:ident : $ret_ty:ty = $conv:expr),* $(,)? }) => {
        #[derive(Clone)]
        #[::pyo3::pyclass(from_py_object)]
        pub struct $py_name {
            inner: ::std::sync::Arc<$swc_ty>,
        }

        impl $py_name {
            pub fn from_owned(node: $swc_ty) -> Self {
                $py_name { inner: ::std::sync::Arc::new(node) }
            }
        }

        #[::pyo3::pymethods]
        impl $py_name {
            $(
                #[getter]
                fn $field(&self, py: ::pyo3::Python<'_>) -> ::pyo3::PyResult<$ret_ty> {
                    $conv(py, self.inner.$field.clone())
                }
            )*
        }
    };
}

pub(crate) use lazy_leaf_node;

macro_rules! py_enum {
    ($py_name:ident, $swc_ty:ty, { $($variant:ident),* $(,)? }) => {
        #[::pyo3::pyclass(eq, eq_int, from_py_object)]
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum $py_name {
            $($variant,)*
        }

        impl ::std::convert::From<$swc_ty> for $py_name {
            fn from(value: $swc_ty) -> Self {
                match value {
                    $(<$swc_ty>::$variant => $py_name::$variant,)*
                }
            }
        }
    };
}

pub(crate) use py_enum;

macro_rules! arc_variant_node {
    ($base:ty, $py_name:ident, $data_enum:ty, $variant:path, $data_ty:ty, { $($field:ident : $ret_ty:ty = $conv:expr),* $(,)? }) => {
        #[::pyo3::pyclass(extends = $base)]
        pub struct $py_name {
            inner: ::std::sync::Arc<$data_enum>,
        }

        impl $py_name {
            pub fn from_arc(inner: ::std::sync::Arc<$data_enum>) -> Self {
                $py_name { inner }
            }

            fn data(&self) -> &$data_ty {
                #[allow(unreachable_patterns)]
                match &*self.inner {
                    $variant(d) => d,
                    _ => unreachable!("data/pyclass variant mismatch"),
                }
            }
        }

        #[::pyo3::pymethods]
        impl $py_name {
            $(
                #[getter]
                fn $field(&self, py: ::pyo3::Python<'_>) -> ::pyo3::PyResult<$ret_ty> {
                    $conv(py, self.data().$field.clone())
                }
            )*
        }
    };
}

pub(crate) use arc_variant_node;
