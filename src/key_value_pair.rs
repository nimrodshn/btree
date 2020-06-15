pub struct KeyValuePair{
    pub key: String,
    pub value: String,
}

impl KeyValuePair {
    new(key: String, value: String) -> KeyValuePair {
        KeyValuePair{
            key,
            value,
        }
    }
}
