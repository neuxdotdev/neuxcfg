use crate::NeuxcfgError;
use std::collections::HashMap;
use toml::Value;
pub fn validate_extra(extra: &HashMap<String, Value>) -> Result<(), NeuxcfgError> {
    for (key, val) in extra {
        if key.starts_with('_') || key.contains('.') {
            return Err(NeuxcfgError::ValidationError(format!(
                "invalid extra key '{}' (must not start with '_' or contain '.')",
                key
            )));
        }
        validate_value_type(val)?;
    }
    Ok(())
}
fn validate_value_type(value: &Value) -> Result<(), NeuxcfgError> {
    match value {
        Value::String(_) | Value::Integer(_) | Value::Float(_) | Value::Boolean(_) => Ok(()),
        Value::Table(table) => {
            for (_k, v) in table {
                validate_value_type(v)?;
            }
            Ok(())
        }
        Value::Array(arr) => {
            for v in arr {
                validate_value_type(v)?;
            }
            Ok(())
        }
        _ => Err(NeuxcfgError::ValidationError(format!(
            "unsupported value type in extra field: {:?}",
            value
        ))),
    }
}
