use crate::{
    conversions::{conv_atom, conv_ctxt, conv_ident, conv_span},
    macros::lazy_leaf_node,
    pyspan::PySpan,
    pytypeinfo::PyTsTypeAnn,
};
use pyo3::prelude::*;
use std::sync::Arc;
use swc_core::ecma::ast::{Ident, IdentName, PrivateName};

lazy_leaf_node!(PyIdentName, IdentName, {
    span: PySpan = conv_span,
    sym: String = conv_atom,
});

lazy_leaf_node!(PyPrivateName, PrivateName, {
    span: PySpan = conv_span,
    name: String = conv_atom,
});

lazy_leaf_node!(PyIdent, Ident, {
    span: PySpan = conv_span,
    ctxt: u32 = conv_ctxt,
    sym: String = conv_atom,
});

#[pyclass]
pub struct PyBindingIdent {
    
    ident: Arc<Ident>,
    #[pyo3(get)]
    pub type_ann: Option<Py<PyTsTypeAnn>>,
}

#[pymethods]
impl PyBindingIdent {
    #[getter]
    fn id(&self, py: Python<'_>) -> PyResult<PyIdent> {
        conv_ident(py, (*self.ident).clone())
    }
}

impl PyBindingIdent {
    pub fn new(ident: Ident, type_ann: Option<Py<PyTsTypeAnn>>) -> Self {
        PyBindingIdent {
            ident: Arc::new(ident),
            type_ann,
        }
    }
}
