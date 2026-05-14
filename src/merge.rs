use toml::Value;
pub fn deep_merge(base: &mut Value, delta: &Value) {
    match (base, delta) {
        (Value::Table(ref mut base_map), Value::Table(ref delta_map)) => {
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
