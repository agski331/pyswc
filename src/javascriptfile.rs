use pyo3::exceptions::PyValueError;
use pyo3::ffi::Py_None;
use pyo3::prelude::*;
use swc_core::common::{FileName, SourceMap, sync::Lrc};
use swc_core::ecma::ast::{EsVersion, Module};
use swc_core::ecma::codegen::{Config, Emitter, text_writer::JsWriter};
use swc_core::ecma::parser::{Parser, StringInput, Syntax, lexer::Lexer};

use crate::conversions::module_to_py;
use crate::pymodule::PySourceModule;

#[pyclass]
pub struct JavaScriptFile {
    script_data: Vec<u8>,
    cm: Lrc<SourceMap>,
    module: Option<Py<PySourceModule>>,
    swc_module: Module,
}

impl JavaScriptFile {
    pub fn new(script_data: &Vec<u8>) -> Option<JavaScriptFile> {
        let script = JavaScriptFile {
            script_data: script_data.clone(),
            cm: Default::default(),
            module: None,
            swc_module: Default::default(),
        };
        return Some(script);
    }

    fn parse(&mut self, py: Python<'_>) -> PyResult<bool> {
        let src_data = String::from_utf8(self.script_data.clone());
        if let Ok(src_string) = src_data {
            let fm = self.cm.as_ref().new_source_file(
                Lrc::new(FileName::Custom("file.js".to_string())),
                src_string,
            );
            let lexer = Lexer::new(
                Syntax::Es(Default::default()),
                EsVersion::latest(),
                StringInput::from(&*fm),
                None,
            );
            let mut parser = Parser::new_from(lexer);
            let parsed_module = parser.parse_module();
            if let Ok(module) = parsed_module {
                self.swc_module = module;

                self.module = Some(module_to_py(py, self.swc_module.clone())?);
                return Ok(true);
            }
        }
        return Ok(false);
    }

    fn emit(&mut self) -> Option<Vec<u8>> {
        let mut buf = Vec::new();
        {
            let writer = JsWriter::new(self.cm.clone(), "\n", &mut buf, None);
            let mut emitter = Emitter {
                cfg: Config::default(),
                cm: self.cm.clone(),
                comments: None,
                wr: writer,
            };
            let emitted_module = emitter.emit_module(&self.swc_module);
            if emitted_module.is_ok() {
                return Some(buf);
            }
        }
        return None;
    }
}

#[pymethods]
impl JavaScriptFile {
    #[new]
    fn py_new(py: Python<'_>, source: String) -> PyResult<Self> {
        let java_opt = JavaScriptFile::new(&source.into_bytes());
        if let Some(mut script) = java_opt {
            let parse_result = script.parse(py)?;
            if !parse_result {
                return Err(PyValueError::new_err("Unable to parse JavaScript"));
            }

            return Ok(script);
        } else {
            return Err(PyValueError::new_err("Failed to parse JavaScript source"));
        }
    }

    #[getter]
    fn module(&self, py: Python<'_>) -> Option<Py<PySourceModule>> {
        if self.module.is_none() {
            return None;
        }

        return Some(self.module.as_ref().unwrap().clone_ref(py));
    }
}
