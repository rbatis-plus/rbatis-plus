use crate::CacheTag;
use std::fmt;

/// A cache key identifying a single cached query result.
///
/// The key is built from:
/// - `namespace`: logical grouping (e.g. table name or mapper namespace)
/// - `sql`: the final, rewritten SQL string
/// - `args`: canonical rbs::Value parameters
/// - `version`: key protocol version for forward compatibility
///
/// The `digest` is computed via [`CacheKey::compute_digest`].
#[derive(Clone)]
pub struct CacheKey {
    pub namespace: String,
    pub sql: String,
    pub args: Vec<rbs::Value>,
    pub version: u8,
    pub digest: u64,
}

impl CacheKey {
    /// Build a cache key from namespace, final SQL and args.
    pub fn new(namespace: impl Into<String>, sql: impl Into<String>, args: Vec<rbs::Value>) -> Self {
        let namespace = namespace.into();
        let sql = sql.into();
        let version = 1;
        let digest = Self::compute_digest(&namespace, &sql, &args, version);
        Self {
            namespace,
            sql,
            args,
            version,
            digest,
        }
    }

    /// Compute a stable 64-bit digest from the key components.
    ///
    /// Uses a simple FNV-1a hash.  This is **not** cryptographically secure;
    /// for threat models involving adversarial key collisions, a MAC/HMAC
    /// should be used instead (planned for a later phase).
    fn compute_digest(namespace: &str, sql: &str, args: &[rbs::Value], version: u8) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
        let mixin = |hash: &mut u64, bytes: &[u8]| {
            for &b in bytes {
                *hash ^= b as u64;
                *hash = hash.wrapping_mul(0x100000001b3); // FNV prime
            }
        };
        mixin(&mut hash, &[version]);
        mixin(&mut hash, namespace.as_bytes());
        mixin(&mut hash, sql.as_bytes());
        // Canonicalise each arg via its Debug representation.
        // This preserves type distinctions (1 vs "1" vs 1.0 vs Null).
        for arg in args {
            let s = format!("{:?}", arg);
            mixin(&mut hash, s.as_bytes());
        }
        hash
    }

    /// The tags associated with this key's namespace.  Currently derived
    /// from the namespace, but may be user-specified in the future.
    pub fn tags(&self) -> Vec<CacheTag> {
        vec![self.namespace.clone()]
    }
}

impl fmt::Debug for CacheKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CacheKey")
            .field("namespace", &self.namespace)
            .field("sql", &self.sql)
            .field("digest", &format_args!("0x{:016x}", self.digest))
            .field("version", &self.version)
            .field("args_len", &self.args.len())
            .finish()
    }
}

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.digest == other.digest
    }
}

impl Eq for CacheKey {}

impl std::hash::Hash for CacheKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.digest.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rbs::Value;

    #[test]
    fn different_args_produce_different_keys() {
        let k1 = CacheKey::new("ns", "select * from t where id = ?", vec![Value::I64(1)]);
        let k2 = CacheKey::new("ns", "select * from t where id = ?", vec![Value::I64(2)]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn different_sql_produces_different_keys() {
        let k1 = CacheKey::new("ns", "select * from t", vec![]);
        let k2 = CacheKey::new("ns", "select * from t where id = 1", vec![]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn int_vs_string_arg_produces_different_keys() {
        let k1 = CacheKey::new("ns", "?", vec![Value::I64(1)]);
        let k2 = CacheKey::new("ns", "?", vec![Value::String("1".into())]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn null_vs_empty_string_produces_different_keys() {
        let k1 = CacheKey::new("ns", "?", vec![Value::Null]);
        let k2 = CacheKey::new("ns", "?", vec![Value::String("".into())]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn same_inputs_produce_same_key() {
        let k1 = CacheKey::new("ns", "select 1", vec![Value::I64(42)]);
        let k2 = CacheKey::new("ns", "select 1", vec![Value::I64(42)]);
        assert_eq!(k1, k2);
    }
}
