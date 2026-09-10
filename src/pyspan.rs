use pyo3::prelude::*;
use swc_core::common::Span;

#[derive(Clone)]
#[pyclass]
pub struct PySpan {
    #[pyo3(get)]
    low_pos: u32,
    #[pyo3(get)]
    high_pos: u32,
}

impl From<Span> for PySpan {
    fn from(span: Span) -> Self {
        PySpan {
            low_pos: span.lo.0,
            high_pos: span.hi.0,
        }
    }
}
