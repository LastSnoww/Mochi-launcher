use std::fmt;

use serde::Deserialize;

/// A token or other credential. It can be deserialized from API responses but never
/// serialized or printed: `Debug` is redacted and there is no `Display`.
#[derive(Clone, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Use only where the raw value has to leave the process (HTTP request, keychain).
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret([redacted])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_output_is_redacted() {
        let secret = Secret::new("eyJhbGciOi.very-secret".into());
        let printed = format!("{secret:?} {:?}", Some(&secret));
        assert!(!printed.contains("very-secret"));
        assert!(printed.contains("[redacted]"));
    }

    #[test]
    fn deserializes_from_plain_string() {
        let secret: Secret = serde_json::from_str("\"abc\"").unwrap();
        assert_eq!(secret.expose(), "abc");
    }
}
