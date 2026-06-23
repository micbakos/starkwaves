use crate::compat::cairo::cairo_value::CairoValue;
use crate::compat::cairo::panic_result::{CairoError, CairoResult};
use cairo_lang_sierra::program::Program;
use cairo_native::Value;
use cairo_native::context::NativeContext;
use cairo_native::executor::JitNativeExecutor;
use cairo_native::utils::find_function_id;
use std::path::Path;

pub struct CairoRunner {
    executor: JitNativeExecutor<'static>,
    sierra_program: Program,
}

impl CairoRunner {
    /// Create a new prover by loading and compiling the Sierra program
    pub fn new(program_path: &Path) -> Self {
        let sierra_json =
            std::fs::read_to_string(program_path).expect("Unable to read sierra program file");

        let program = serde_json::from_str::<Program>(&sierra_json)
            .expect("Unable to parse sierra program file");

        // Instantiate a Cairo Native MLIR context
        // Leak it to make it 'static (it lives for the entire program)
        let native_context = Box::leak(Box::new(NativeContext::new()));

        // Compile the sierra program into a MLIR module
        let native_module = native_context
            .compile(&program, false, Some(Default::default()), None)
            .expect("Unable to compile native module");

        // Instantiate the JIT executor (consumes the module)
        let executor = JitNativeExecutor::from_native_module(native_module, Default::default())
            .expect("Unable to create executor");

        Self {
            executor,
            sierra_program: program,
        }
    }

    pub fn execute_cairo_fn(&self, selector: &str, args: Vec<Value>) -> CairoResult {
        let function_id = find_function_id(&self.sierra_program, selector)
            .expect(format!("Could not find function {}", selector).as_str());

        let gas_limit = Some(u64::MAX);

        let value = self
            .executor
            .invoke_dynamic(function_id, &args, gas_limit)
            .map_err(|e| CairoError::from_error(e))
            .map(|exec| CairoValue::from(exec))?;

        CairoResult::from(value)
    }
}
