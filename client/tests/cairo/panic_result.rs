use crate::cairo::cairo_value::CairoValue;
use cairo_native::Value;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CairoError {
    values: Vec<Value>,
    message: Option<String>,
}

pub type CairoResult = Result<Value, CairoError>;

impl CairoError {
    pub fn from_values(values: impl IntoIterator<Item = Value>, message: &str) -> CairoError {
        CairoError {
            values: values.into_iter().collect(),
            message: Some(message.into()),
        }
    }

    pub fn from_error(error: cairo_native::error::Error) -> CairoError {
        CairoError {
            values: vec![],
            message: Some(format!("{:?}", error)),
        }
    }
}

impl Display for CairoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message.as_ref().unwrap_or(&String::new()))
    }
}

impl From<CairoValue> for CairoResult {
    fn from(value: CairoValue) -> Self {
        match value.0 {
            Value::Enum { tag, value, .. } => {
                if tag == 0 {
                    match *value {
                        Value::Struct { ref fields, .. } => {
                            if let Some(value) = fields.first() {
                                Ok(value.clone())
                            } else {
                                Err(CairoError::from_values(
                                    fields.clone(),
                                    "Found Ok result but could not extract values",
                                ))
                            }
                        }
                        _ => Err(CairoError::from_values(
                            vec![*value],
                            "Found ok result but returned value was not a struct",
                        ))
                    }
                } else if tag == 1 {
                    decode_cairo_panic(value.as_ref())
                        .map(|e| Err(e))
                        .unwrap_or(Err(CairoError::from_values(
                            vec![*value],
                            "Could not decode error."
                        )))
                } else {
                    Err(CairoError::from_values(vec![*value], format!("Tag for enum was neither 0 (ok) or 1 (Err). It was {}", tag).as_ref()))
                }
            }
            _ => Err(CairoError::from_values(
                vec![value.0],
                "Cairo result was not an enum (PanicResult)."
            ))
        }
    }
}

fn decode_cairo_panic(panic_struct: &Value) -> Option<CairoError> {
    match panic_struct {
        Value::Struct { fields, .. } => {
            if fields.len() >= 2 {
                if let Value::Array(panic_data) = &fields[1] {
                    return decode_error(panic_data)
                }
            }
        }
        _ => {}
    }

    None
}


fn decode_error(felts: &[Value]) -> Option<CairoError> {
    let mut message = String::new();

    for value in felts {
        if let Value::Felt252(felt) = value {
            // Convert felt252 to bytes
            let bytes = felt.to_bytes_be();

            // Skip leading zeros and decode as UTF-8
            let decoded = bytes.iter()
                .skip_while(|&&b| b == 0)
                .copied()
                .collect::<Vec<u8>>();

            if let Ok(s) = String::from_utf8(decoded) {
                if !s.is_empty() && !s.chars().all(|c| c.is_control()) {
                    message.push_str(&s);
                }
            }
        }
    }

    // Fallback: show raw felt values if decode failed
    if message.is_empty() {
        return None
    }

    Some(CairoError::from_values(felts.iter().cloned(), message.as_ref()))
}
