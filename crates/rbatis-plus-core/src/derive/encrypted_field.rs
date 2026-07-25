#[derive(Debug, Clone)]
pub struct EncryptedFieldAttr {
    pub column: &'static str,
    pub algorithm: &'static str,
    pub encrypt: bool,
    pub decrypt: bool,
}

impl Default for EncryptedFieldAttr {
    fn default() -> Self {
        Self { column: "", algorithm: "AES", encrypt: true, decrypt: true }
    }
}

pub trait EncryptedField {
    fn encrypted_fields() -> Vec<EncryptedFieldAttr>;
}
