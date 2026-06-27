//! Simple internationalization (i18n) support for Jambon.
//!
//! Stores translations as composite `"{lang}.{key}"` → string pairs and falls
//! back to English when a requested translation is missing.
//!
//! ## Supported languages
//!
//! - `en` — English (default)
//! - `fr` — French
//!
//! ## Usage
//!
//! ```rust
//! use jambon_bot_i18n::I18n;
//!
//! let i18n = I18n::new();
//! assert_eq!(i18n.tr("en", "hello"), "Hello");
//! assert_eq!(i18n.tr("fr", "hello"), "Bonjour");
//! assert_eq!(i18n.tr("de", "hello"), "Hello"); // falls back to en
//! ```

use std::collections::HashMap;

/// A simple i18n store.
///
/// Falls back to the English translation if the requested language does not
/// have the key. Returns the key itself if no translation exists at all.
#[derive(Clone, Debug)]
pub struct I18n {
    /// Map of `"{lang}.{key}"` → translated string.
    strings: HashMap<&'static str, &'static str>,
}

impl I18n {
    /// Build an [`I18n`] store pre-populated with all built-in translations.
    ///
    /// # Built-in strings
    ///
    /// | Key              | EN               | FR                   |
    /// |------------------|------------------|----------------------|
    /// | `hello`          | Hello            | Bonjour              |
    /// | `goodbye`        | Goodbye          | Au revoir            |
    /// | `yes`            | Yes              | Oui                  |
    /// | `no`             | No               | Non                  |
    /// | `running`        | Running          | En cours             |
    /// | `stopped`        | Stopped          | Arrêté               |
    /// | `error`          | Error            | Erreur               |
    /// | `success`        | Success          | Succès               |
    /// | `loading`        | Loading...       | Chargement...        |
    /// | `command_list`   | List             | Liste                |
    /// | `command_status` | Status           | Statut               |
    /// | `command_start`  | Start            | Démarrer             |
    /// | `command_stop`   | Stop             | Arrêter              |
    /// | `powered_by`     | Powered by       | Propulsé par         |
    #[must_use]
    pub fn new() -> Self {
        let mut strings = HashMap::new();

        // English
        for (key, val) in Self::en_pairs() {
            strings.insert(key, val);
        }

        // French
        for (key, val) in Self::fr_pairs() {
            strings.insert(key, val);
        }

        Self { strings }
    }

    /// Look up a translation for `key` in `lang`.
    ///
    /// Falls back to English if the key is not found for the requested
    /// language. Returns the key itself if no translation exists at all.
    pub fn tr<'a>(&self, lang: &str, key: &'a str) -> &'a str {
        let lookup = composite(lang, key);
        if let Some(val) = self.strings.get(lookup.as_str()) {
            return val;
        }
        let fallback = composite("en", key);
        if let Some(val) = self.strings.get(fallback.as_str()) {
            return val;
        }
        key
    }

    fn en_pairs() -> [(&'static str, &'static str); 14] {
        [
            ("en.hello", "Hello"),
            ("en.goodbye", "Goodbye"),
            ("en.yes", "Yes"),
            ("en.no", "No"),
            ("en.running", "Running"),
            ("en.stopped", "Stopped"),
            ("en.error", "Error"),
            ("en.success", "Success"),
            ("en.loading", "Loading..."),
            ("en.command_list", "List"),
            ("en.command_status", "Status"),
            ("en.command_start", "Start"),
            ("en.command_stop", "Stop"),
            ("en.powered_by", "Powered by"),
        ]
    }

    fn fr_pairs() -> [(&'static str, &'static str); 14] {
        [
            ("fr.hello", "Bonjour"),
            ("fr.goodbye", "Au revoir"),
            ("fr.yes", "Oui"),
            ("fr.no", "Non"),
            ("fr.running", "En cours"),
            ("fr.stopped", "Arrêté"),
            ("fr.error", "Erreur"),
            ("fr.success", "Succès"),
            ("fr.loading", "Chargement..."),
            ("fr.command_list", "Liste"),
            ("fr.command_status", "Statut"),
            ("fr.command_start", "Démarrer"),
            ("fr.command_stop", "Arrêter"),
            ("fr.powered_by", "Propulsé par"),
        ]
    }
}

fn composite(lang: &str, key: &str) -> String {
    let mut s = String::with_capacity(lang.len() + 1 + key.len());
    s.push_str(lang);
    s.push('.');
    s.push_str(key);
    s
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_translation() {
        let i18n = I18n::new();
        assert_eq!(i18n.tr("en", "hello"), "Hello");
        assert_eq!(i18n.tr("en", "goodbye"), "Goodbye");
    }

    #[test]
    fn test_french_translation() {
        let i18n = I18n::new();
        assert_eq!(i18n.tr("fr", "hello"), "Bonjour");
        assert_eq!(i18n.tr("fr", "goodbye"), "Au revoir");
    }

    #[test]
    fn test_fallback_to_english() {
        let i18n = I18n::new();
        assert_eq!(i18n.tr("de", "hello"), "Hello");
    }

    #[test]
    fn test_unknown_key_returns_key() {
        let i18n = I18n::new();
        assert_eq!(i18n.tr("en", "nonexistent_key"), "nonexistent_key");
    }

    #[test]
    fn test_all_english_keys_present() {
        let i18n = I18n::new();
        let keys = [
            "hello",
            "goodbye",
            "yes",
            "no",
            "running",
            "stopped",
            "error",
            "success",
            "loading",
            "command_list",
            "command_status",
            "command_start",
            "command_stop",
            "powered_by",
        ];
        for key in &keys {
            assert_ne!(i18n.tr("en", key), *key, "key '{key}' should have an EN translation");
        }
    }

    #[test]
    fn test_all_french_keys_present() {
        let i18n = I18n::new();
        let keys = [
            "hello",
            "goodbye",
            "yes",
            "no",
            "running",
            "stopped",
            "error",
            "success",
            "loading",
            "command_list",
            "command_status",
            "command_start",
            "command_stop",
            "powered_by",
        ];
        for key in &keys {
            assert_ne!(
                i18n.tr("fr", key),
                *key,
                "key '{key}' should have an FR translation (or EN fallback)"
            );
        }
    }

    #[test]
    fn test_default_is_new() {
        let a = I18n::new();
        let b = I18n::default();
        assert_eq!(a.tr("en", "hello"), b.tr("en", "hello"));
    }
}
