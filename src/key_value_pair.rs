pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

impl KeyValuePair {
    pub fn new(key: String, value: String) -> KeyValuePair {
        KeyValuePair { key, value }
    }
}

impl Clone for KeyValuePair {
    fn clone(&self) -> KeyValuePair {
        KeyValuePair {
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }
}
