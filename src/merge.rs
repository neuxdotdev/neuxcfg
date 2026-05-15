use toml::Value;

/// Recursively merges `delta` into `base`, modifying `base` in place.
///
/// - If both `base` and `delta` are tables, each key from `delta` is merged
///   recursively; keys only in `delta` are inserted.
/// - For any other value type (string, integer, array, etc.), `base` is
///   replaced by a clone of `delta`.
///
/// This is used to apply partial configuration updates without overwriting
/// the entire TOML document.
///
/// # Examples
///
/// ```rust
/// use toml::toml;
/// use neuxcfg::merge::deep_merge;
///
/// let mut base = toml! {
///     [project]
///     name = "old"
///     [limits]
///     max = 10
/// };
/// let delta = toml! {
///     [project]
///     name = "new"
///     [limits]
///     min = 1
/// };
/// deep_merge(&mut base, &delta);
/// let expected = toml! {
///     [project]
///     name = "new"
///     [limits]
///     max = 10
///     min = 1
/// };
/// assert_eq!(base, expected);
/// ```
pub fn deep_merge(base: &mut Value, delta: &Value) {
    match (base, delta) {
        (Value::Table(base_map), Value::Table(delta_map)) => {
            for (key, val) in delta_map {
                match base_map.get_mut(key) {
                    Some(existing) => deep_merge(existing, val),
                    None => {
                        base_map.insert(key.clone(), val.clone());
                    }
                }
            }
        }
        (base_val, delta_val) => *base_val = delta_val.clone(),
    }
}