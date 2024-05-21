use std::env;

pub fn get_env_var(key: &String) -> Option<String> {
    match env::var(key) {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}

