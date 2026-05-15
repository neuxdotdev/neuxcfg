use crate::NeuxcfgError;
use std::collections::HashMap;
use toml::Value;

/// Validates extra fields in a project configuration.
///
/// Checks that:
/// - Keys do not start with `_` and do not contain `.`.
/// - Values are of an allowed TOML type: string, integer, float, boolean,
///   table, or array of those types. (Date/time values are rejected.)
///
/// This function is called before writing any project configuration.
///
/// # Errors
///
/// Returns [`NeuxcfgError::ValidationError`] with a descriptive message if
/// a key or value is invalid.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use toml::Value;
/// use neuxcfg::validate::validate_extra;
///
/// let mut extra = HashMap::new();
/// extra.insert("allowed_key".into(), Value::String("ok".into()));
/// assert!(validate_extra(&extra).is_ok());
///
/// extra.insert("_bad".into(), Value::Boolean(true));
/// assert!(validate_extra(&extra).is_err());
/// ```
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
