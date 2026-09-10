
use swc_core::common::{sync::Lrc, FileName, SourceMap};
use swc_core::ecma::ast::{EsVersion, Module};
use swc_core::ecma::parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_core::ecma::codegen::{text_writer::JsWriter, Config, Emitter};

pub struct JavaScriptFile{
    script_data: Vec<u8>,
    cm: Lrc<SourceMap>,
    module: Module

}

impl JavaScriptFile{
    pub fn new(script_data: &Vec<u8>) -> Option<JavaScriptFile>{
        let mut script = JavaScriptFile{script_data: script_data.clone(), cm: Default::default(), module: Default::default()};

        if !script.parse(){
            return None;
        }
        return Some(script);
    }

    fn parse(&mut self) -> bool{
        let src_data = String::from_utf8(self.script_data.clone());
        if let Ok(src_string) = src_data{
            let fm = self.cm.as_ref().new_source_file(Lrc::new(FileName::Custom("file.js".to_string())), src_string);
            let lexer = Lexer::new(Syntax::Es(Default::default()), EsVersion::latest(), StringInput::from(&*fm), None);
            let mut parser = Parser::new_from(lexer);
            let parsed_module = parser.parse_module();
            if let Ok(module) = parsed_module{
                self.module = module;
                return true;
            }
        }
        return false;
    }

    fn emit(&mut self) -> Option<Vec<u8>>{
        let mut buf = Vec::new();
        {
            let writer = JsWriter::new(self.cm.clone(), "\n", &mut buf, None);
            let mut emitter = Emitter{
                cfg: Config::default(),
                cm: self.cm.clone(),
                comments: None,
                wr: writer
            };
            let emitted_module = emitter.emit_module(&self.module);
            if emitted_module.is_ok(){
                return Some(buf);
            }
        }
        return None;
    }


}