use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Vocabulary entity representing an English word with Japanese translation and examples
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vocabulary {
    pub id: i32,
    pub en_word: String,
    pub ja_word: String,
    pub en_example: Option<String>,
    pub ja_example: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request structure for creating a new vocabulary entry
#[derive(Debug, Deserialize)]
pub struct CreateVocabularyRequest {
    pub en_word: String,
    pub ja_word: String,
    pub en_example: Option<String>,
    pub ja_example: Option<String>,
}

impl CreateVocabularyRequest {
    /// Validate the create vocabulary request
    pub fn validate(&self) -> Result<(), String> {
        // Validate en_word (required)
        if self.en_word.trim().is_empty() {
            return Err("English word cannot be empty".to_string());
        }
        
        if self.en_word.len() > 200 {
            return Err("English word cannot exceed 200 characters".to_string());
        }

        // Validate ja_word (required)
        if self.ja_word.trim().is_empty() {
            return Err("Japanese word cannot be empty".to_string());
        }
        
        if self.ja_word.len() > 200 {
            return Err("Japanese word cannot exceed 200 characters".to_string());
        }

        // Validate en_example if provided (optional)
        if let Some(ref example) = self.en_example {
            if example.len() > 1000 {
                return Err("English example cannot exceed 1000 characters".to_string());
            }
        }

        // Validate ja_example if provided (optional)
        if let Some(ref example) = self.ja_example {
            if example.len() > 1000 {
                return Err("Japanese example cannot exceed 1000 characters".to_string());
            }
        }

        Ok(())
    }

    /// Get normalized en_word (trimmed)
    pub fn get_normalized_en_word(&self) -> String {
        self.en_word.trim().to_string()
    }

    /// Get normalized ja_word (trimmed)
    pub fn get_normalized_ja_word(&self) -> String {
        self.ja_word.trim().to_string()
    }

    /// Get normalized en_example (trimmed, None if empty)
    pub fn get_normalized_en_example(&self) -> Option<String> {
        self.en_example
            .as_ref()
            .map(|e| e.trim().to_string())
            .filter(|e| !e.is_empty())
    }

    /// Get normalized ja_example (trimmed, None if empty)
    pub fn get_normalized_ja_example(&self) -> Option<String> {
        self.ja_example
            .as_ref()
            .map(|e| e.trim().to_string())
            .filter(|e| !e.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vocabulary_request_validation() {
        // Valid request with examples
        let valid_request = CreateVocabularyRequest {
            en_word: "hello".to_string(),
            ja_word: "こんにちは".to_string(),
            en_example: Some("Hello, how are you?".to_string()),
            ja_example: Some("こんにちは、お元気ですか？".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        // Valid request without examples
        let valid_request_no_examples = CreateVocabularyRequest {
            en_word: "hello".to_string(),
            ja_word: "こんにちは".to_string(),
            en_example: None,
            ja_example: None,
        };
        assert!(valid_request_no_examples.validate().is_ok());

        // Empty en_word
        let invalid_en_word = CreateVocabularyRequest {
            en_word: "".to_string(),
            ja_word: "こんにちは".to_string(),
            en_example: None,
            ja_example: None,
        };
        assert!(invalid_en_word.validate().is_err());

        // Empty ja_word
        let invalid_ja_word = CreateVocabularyRequest {
            en_word: "hello".to_string(),
            ja_word: "".to_string(),
            en_example: None,
            ja_example: None,
        };
        assert!(invalid_ja_word.validate().is_err());

        // en_word too long
        let long_en_word = CreateVocabularyRequest {
            en_word: "a".repeat(201),
            ja_word: "こんにちは".to_string(),
            en_example: None,
            ja_example: None,
        };
        assert!(long_en_word.validate().is_err());

        // ja_word too long
        let long_ja_word = CreateVocabularyRequest {
            en_word: "hello".to_string(),
            ja_word: "あ".repeat(201),
            en_example: None,
            ja_example: None,
        };
        assert!(long_ja_word.validate().is_err());

        // en_example too long
        let long_en_example = CreateVocabularyRequest {
            en_word: "hello".to_string(),
            ja_word: "こんにちは".to_string(),
            en_example: Some("a".repeat(1001)),
            ja_example: None,
        };
        assert!(long_en_example.validate().is_err());

        // ja_example too long
        let long_ja_example = CreateVocabularyRequest {
            en_word: "hello".to_string(),
            ja_word: "こんにちは".to_string(),
            en_example: None,
            ja_example: Some("あ".repeat(1001)),
        };
        assert!(long_ja_example.validate().is_err());
    }

    #[test]
    fn test_create_vocabulary_request_normalization() {
        let request = CreateVocabularyRequest {
            en_word: "  hello  ".to_string(),
            ja_word: "  こんにちは  ".to_string(),
            en_example: Some("  Hello, how are you?  ".to_string()),
            ja_example: Some("   ".to_string()), // Only whitespace
        };
        
        assert_eq!(request.get_normalized_en_word(), "hello");
        assert_eq!(request.get_normalized_ja_word(), "こんにちは");
        assert_eq!(request.get_normalized_en_example(), Some("Hello, how are you?".to_string()));
        assert_eq!(request.get_normalized_ja_example(), None); // Empty should be None
    }

    #[test]
    fn test_vocabulary_serialization() {
        let vocabulary = Vocabulary {
            id: 1,
            en_word: "hello".to_string(),
            ja_word: "こんにちは".to_string(),
            en_example: Some("Hello, how are you?".to_string()),
            ja_example: Some("こんにちは、お元気ですか？".to_string()),
            created_at: DateTime::parse_from_rfc3339("2022-01-01T00:00:00Z").unwrap().with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339("2022-01-01T00:00:00Z").unwrap().with_timezone(&Utc),
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&vocabulary).expect("Failed to serialize vocabulary");
        let expected = r#"{"id":1,"en_word":"hello","ja_word":"こんにちは","en_example":"Hello, how are you?","ja_example":"こんにちは、お元気ですか？","created_at":"2022-01-01T00:00:00Z","updated_at":"2022-01-01T00:00:00Z"}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn test_vocabulary_serialization_without_examples() {
        let vocabulary = Vocabulary {
            id: 1,
            en_word: "hello".to_string(),
            ja_word: "こんにちは".to_string(),
            en_example: None,
            ja_example: None,
            created_at: DateTime::parse_from_rfc3339("2022-01-01T00:00:00Z").unwrap().with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339("2022-01-01T00:00:00Z").unwrap().with_timezone(&Utc),
        };

        // Test serialization to JSON with null examples
        let json = serde_json::to_string(&vocabulary).expect("Failed to serialize vocabulary");
        let expected = r#"{"id":1,"en_word":"hello","ja_word":"こんにちは","en_example":null,"ja_example":null,"created_at":"2022-01-01T00:00:00Z","updated_at":"2022-01-01T00:00:00Z"}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn test_vocabulary_deserialization() {
        let json = r#"{"id":1,"en_word":"hello","ja_word":"こんにちは","en_example":"Hello, how are you?","ja_example":"こんにちは、お元気ですか？","created_at":"2022-01-01T00:00:00Z","updated_at":"2022-01-01T00:00:00Z"}"#;
        
        // Test deserialization from JSON
        let vocabulary: Vocabulary = serde_json::from_str(json).expect("Failed to deserialize vocabulary");
        
        assert_eq!(vocabulary.id, 1);
        assert_eq!(vocabulary.en_word, "hello");
        assert_eq!(vocabulary.ja_word, "こんにちは");
        assert_eq!(vocabulary.en_example, Some("Hello, how are you?".to_string()));
        assert_eq!(vocabulary.ja_example, Some("こんにちは、お元気ですか？".to_string()));
        assert_eq!(vocabulary.created_at, DateTime::parse_from_rfc3339("2022-01-01T00:00:00Z").unwrap().with_timezone(&Utc));
        assert_eq!(vocabulary.updated_at, DateTime::parse_from_rfc3339("2022-01-01T00:00:00Z").unwrap().with_timezone(&Utc));
    }

    #[test]
    fn test_vocabulary_deserialization_without_examples() {
        let json = r#"{"id":1,"en_word":"hello","ja_word":"こんにちは","en_example":null,"ja_example":null,"created_at":"2022-01-01T00:00:00Z","updated_at":"2022-01-01T00:00:00Z"}"#;
        
        // Test deserialization from JSON with null examples
        let vocabulary: Vocabulary = serde_json::from_str(json).expect("Failed to deserialize vocabulary");
        
        assert_eq!(vocabulary.id, 1);
        assert_eq!(vocabulary.en_word, "hello");
        assert_eq!(vocabulary.ja_word, "こんにちは");
        assert_eq!(vocabulary.en_example, None);
        assert_eq!(vocabulary.ja_example, None);
        assert_eq!(vocabulary.created_at, DateTime::parse_from_rfc3339("2022-01-01T00:00:00Z").unwrap().with_timezone(&Utc));
        assert_eq!(vocabulary.updated_at, DateTime::parse_from_rfc3339("2022-01-01T00:00:00Z").unwrap().with_timezone(&Utc));
    }

    #[test]
    fn test_create_vocabulary_request_deserialization() {
        // Test with examples
        let json_with_examples = r#"{"en_word":"hello","ja_word":"こんにちは","en_example":"Hello, how are you?","ja_example":"こんにちは、お元気ですか？"}"#;
        let request: CreateVocabularyRequest = serde_json::from_str(json_with_examples).expect("Failed to deserialize CreateVocabularyRequest");
        
        assert_eq!(request.en_word, "hello");
        assert_eq!(request.ja_word, "こんにちは");
        assert_eq!(request.en_example, Some("Hello, how are you?".to_string()));
        assert_eq!(request.ja_example, Some("こんにちは、お元気ですか？".to_string()));

        // Test without examples
        let json_without_examples = r#"{"en_word":"hello","ja_word":"こんにちは"}"#;
        let request: CreateVocabularyRequest = serde_json::from_str(json_without_examples).expect("Failed to deserialize CreateVocabularyRequest");
        
        assert_eq!(request.en_word, "hello");
        assert_eq!(request.ja_word, "こんにちは");
        assert_eq!(request.en_example, None);
        assert_eq!(request.ja_example, None);

        // Test with null examples
        let json_null_examples = r#"{"en_word":"hello","ja_word":"こんにちは","en_example":null,"ja_example":null}"#;
        let request: CreateVocabularyRequest = serde_json::from_str(json_null_examples).expect("Failed to deserialize CreateVocabularyRequest");
        
        assert_eq!(request.en_word, "hello");
        assert_eq!(request.ja_word, "こんにちは");
        assert_eq!(request.en_example, None);
        assert_eq!(request.ja_example, None);
    }
}
