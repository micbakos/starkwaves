use cairo_native::Value;
use cairo_native::execution_result::ExecutionResult;

#[derive(Debug)]
pub struct CairoValue(pub Value);
impl CairoValue {
    pub fn from(result: ExecutionResult) -> CairoValue {
        CairoValue(result.return_value)
    }
}
