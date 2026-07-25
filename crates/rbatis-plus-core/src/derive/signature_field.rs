#[derive(Debug, Clone)]
pub struct SignatureFieldAttr {
    pub property: &'static str,
    pub column: &'static str,
    pub order: u32,
    pub stored: bool,
}

impl Default for SignatureFieldAttr {
    fn default() -> Self {
        Self { property: "", column: "", order: 0, stored: false }
    }
}

pub trait SignatureField {
    fn signature_fields() -> Vec<SignatureFieldAttr>;
}
