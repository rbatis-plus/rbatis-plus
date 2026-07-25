pub trait TableSignature {
    fn signature_enabled() -> bool { true }
    fn union_all() -> bool { false }
}
