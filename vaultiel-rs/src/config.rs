//! Task configuration for generic emoji metadata fields.

use serde::{Deserialize, Serialize};

/// Value type for an emoji metadata field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EmojiValueType {
    /// ISO date (YYYY-MM-DD) after emoji.
    Date,
    /// Single word/token after emoji.
    String,
    /// Multi-word text until next registered emoji.
    Text,
    /// Numeric value after emoji.
    Number,
    /// No inline value; presence sets field_name to the predefined value.
    Flag { value: std::string::String },
    /// No inline value; presence sets a shared field_name to the predefined value.
    Enum { value: std::string::String },
}

/// Definition of a single emoji metadata field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmojiFieldDef {
    /// The emoji character(s) used in task text.
    pub emoji: std::string::String,
    /// The field name this maps to in the metadata map.
    pub field_name: std::string::String,
    /// How to parse the value after the emoji.
    pub value_type: EmojiValueType,
    /// Sort order for output (lower = earlier in formatted task).
    pub order: u32,
}

/// Task configuration: defines all emoji metadata fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    /// Emoji field definitions.
    pub fields: Vec<EmojiFieldDef>,
}

impl TaskConfig {
    /// Create an empty task config with no fields.
    pub fn empty() -> Self {
        Self {
            fields: Vec::new(),
        }
    }

    /// Return fields sorted by order.
    pub fn sorted_fields(&self) -> Vec<&EmojiFieldDef> {
        let mut sorted: Vec<&EmojiFieldDef> = self.fields.iter().collect();
        sorted.sort_by_key(|f| f.order);
        sorted
    }

    /// Get all registered emoji strings (for detecting "next emoji" boundaries).
    pub fn all_emojis(&self) -> Vec<&str> {
        self.fields.iter().map(|f| f.emoji.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_config() {
        let config = TaskConfig::empty();
        assert!(config.fields.is_empty());
        assert!(config.sorted_fields().is_empty());
        assert!(config.all_emojis().is_empty());
    }

    #[test]
    fn test_sorted_fields() {
        let config = TaskConfig {
            fields: vec![
                EmojiFieldDef {
                    emoji: "📅".to_string(),
                    field_name: "due".to_string(),
                    value_type: EmojiValueType::Date,
                    order: 20,
                },
                EmojiFieldDef {
                    emoji: "🆔".to_string(),
                    field_name: "id".to_string(),
                    value_type: EmojiValueType::String,
                    order: 10,
                },
            ],
        };

        let sorted = config.sorted_fields();
        assert_eq!(sorted[0].field_name, "id");
        assert_eq!(sorted[1].field_name, "due");
    }

    #[test]
    fn test_all_emojis() {
        let config = TaskConfig {
            fields: vec![
                EmojiFieldDef {
                    emoji: "📅".to_string(),
                    field_name: "due".to_string(),
                    value_type: EmojiValueType::Date,
                    order: 1,
                },
                EmojiFieldDef {
                    emoji: "⏫".to_string(),
                    field_name: "priority".to_string(),
                    value_type: EmojiValueType::Flag { value: "high".to_string() },
                    order: 2,
                },
            ],
        };

        let emojis = config.all_emojis();
        assert_eq!(emojis.len(), 2);
        assert!(emojis.contains(&"📅"));
        assert!(emojis.contains(&"⏫"));
    }
}
