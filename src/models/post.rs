use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

/// Post entity representing a content item created by a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub content: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Request structure for creating a new post
#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub user_id: String,
    pub title: String,
    pub content: Option<String>,
}

impl Post {
    /// Create a new Post instance with generated ID and timestamps
    pub fn new(user_id: String, title: String, content: Option<String>) -> Self {
        let now = Utc::now().timestamp();
        
        Post {
            id: Uuid::new_v4().to_string(),
            user_id,
            title,
            content,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update post fields and refresh updated_at timestamp
    pub fn update(&mut self, title: Option<String>, content: Option<Option<String>>) {
        if let Some(new_title) = title {
            self.title = new_title;
        }
        
        if let Some(new_content) = content {
            self.content = new_content;
        }
        
        self.updated_at = Utc::now().timestamp();
    }
}

impl CreatePostRequest {
    /// Validate the create post request
    pub fn validate(&self) -> Result<(), String> {
        // Validate user_id
        if self.user_id.trim().is_empty() {
            return Err("User ID cannot be empty".to_string());
        }
        
        // Validate UUID format for user_id
        if Uuid::parse_str(&self.user_id).is_err() {
            return Err("User ID must be a valid UUID".to_string());
        }

        // Validate title
        if self.title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }
        
        if self.title.len() > 200 {
            return Err("Title cannot exceed 200 characters".to_string());
        }

        // Validate content if provided
        if let Some(ref content) = self.content {
            if content.len() > 10000 {
                return Err("Content cannot exceed 10000 characters".to_string());
            }
        }

        Ok(())
    }

    /// Convert to Post entity
    pub fn into_post(self) -> Post {
        let normalized_content = self.content
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty());
            
        Post::new(
            self.user_id.trim().to_string(),
            self.title.trim().to_string(),
            normalized_content,
        )
    }

    /// Get normalized title (trimmed)
    pub fn get_normalized_title(&self) -> String {
        self.title.trim().to_string()
    }

    /// Get normalized content (trimmed, None if empty)
    pub fn get_normalized_content(&self) -> Option<String> {
        self.content
            .as_ref()
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
    }
}

/// Validate UUID string format
pub fn is_valid_uuid(uuid_str: &str) -> bool {
    Uuid::parse_str(uuid_str).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_creation() {
        let user_id = Uuid::new_v4().to_string();
        let post = Post::new(
            user_id.clone(),
            "Test Title".to_string(),
            Some("Test content".to_string()),
        );
        
        assert!(!post.id.is_empty());
        assert_eq!(post.user_id, user_id);
        assert_eq!(post.title, "Test Title");
        assert_eq!(post.content, Some("Test content".to_string()));
        assert!(post.created_at > 0);
        assert_eq!(post.created_at, post.updated_at);
    }

    #[test]
    fn test_post_creation_without_content() {
        let user_id = Uuid::new_v4().to_string();
        let post = Post::new(
            user_id.clone(),
            "Test Title".to_string(),
            None,
        );
        
        assert!(!post.id.is_empty());
        assert_eq!(post.user_id, user_id);
        assert_eq!(post.title, "Test Title");
        assert_eq!(post.content, None);
    }

    #[test]
    fn test_post_update() {
        let user_id = Uuid::new_v4().to_string();
        let mut post = Post::new(
            user_id,
            "Original Title".to_string(),
            Some("Original content".to_string()),
        );
        
        let original_created_at = post.created_at;
        let original_updated_at = post.updated_at;
        
        // Sleep for 1 second to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        post.update(
            Some("Updated Title".to_string()),
            Some(Some("Updated content".to_string())),
        );
        
        assert_eq!(post.title, "Updated Title");
        assert_eq!(post.content, Some("Updated content".to_string()));
        assert_eq!(post.created_at, original_created_at);
        assert!(post.updated_at > original_updated_at);
    }

    #[test]
    fn test_create_post_request_validation() {
        let user_id = Uuid::new_v4().to_string();
        
        // Valid request with content
        let valid_request = CreatePostRequest {
            user_id: user_id.clone(),
            title: "Test Title".to_string(),
            content: Some("Test content".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        // Valid request without content
        let valid_request_no_content = CreatePostRequest {
            user_id: user_id.clone(),
            title: "Test Title".to_string(),
            content: None,
        };
        assert!(valid_request_no_content.validate().is_ok());

        // Empty user_id
        let invalid_user_id = CreatePostRequest {
            user_id: "".to_string(),
            title: "Test Title".to_string(),
            content: None,
        };
        assert!(invalid_user_id.validate().is_err());

        // Invalid UUID format for user_id
        let invalid_uuid = CreatePostRequest {
            user_id: "not-a-uuid".to_string(),
            title: "Test Title".to_string(),
            content: None,
        };
        assert!(invalid_uuid.validate().is_err());

        // Empty title
        let invalid_title = CreatePostRequest {
            user_id: user_id.clone(),
            title: "".to_string(),
            content: None,
        };
        assert!(invalid_title.validate().is_err());

        // Title too long
        let long_title = CreatePostRequest {
            user_id: user_id.clone(),
            title: "a".repeat(201),
            content: None,
        };
        assert!(long_title.validate().is_err());

        // Content too long
        let long_content = CreatePostRequest {
            user_id,
            title: "Test Title".to_string(),
            content: Some("a".repeat(10001)),
        };
        assert!(long_content.validate().is_err());
    }

    #[test]
    fn test_create_post_request_into_post() {
        let user_id = Uuid::new_v4().to_string();
        let request = CreatePostRequest {
            user_id: user_id.clone(),
            title: "  Test Title  ".to_string(),
            content: Some("  Test content  ".to_string()),
        };
        
        let post = request.into_post();
        
        assert_eq!(post.user_id, user_id);
        assert_eq!(post.title, "Test Title");
        assert_eq!(post.content, Some("Test content".to_string()));
    }

    #[test]
    fn test_create_post_request_normalization() {
        let user_id = Uuid::new_v4().to_string();
        let request = CreatePostRequest {
            user_id: user_id.clone(),
            title: "  Test Title  ".to_string(),
            content: Some("   ".to_string()), // Only whitespace
        };
        
        let post = request.into_post();
        
        assert_eq!(post.title, "Test Title");
        assert_eq!(post.content, None); // Empty content should be None
    }

    #[test]
    fn test_uuid_validation() {
        let valid_uuid = Uuid::new_v4().to_string();
        assert!(is_valid_uuid(&valid_uuid));
        
        assert!(!is_valid_uuid("not-a-uuid"));
        assert!(!is_valid_uuid(""));
        assert!(!is_valid_uuid("123"));
    }

    #[test]
    fn test_post_serialization() {
        let post = Post {
            id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            user_id: "987fcdeb-51a2-43d1-9f12-345678901234".to_string(),
            title: "Test Post".to_string(),
            content: Some("This is test content".to_string()),
            created_at: 1640995200,
            updated_at: 1640995200,
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&post).expect("Failed to serialize post");
        let expected = r#"{"id":"123e4567-e89b-12d3-a456-426614174000","user_id":"987fcdeb-51a2-43d1-9f12-345678901234","title":"Test Post","content":"This is test content","created_at":1640995200,"updated_at":1640995200}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn test_post_serialization_without_content() {
        let post = Post {
            id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            user_id: "987fcdeb-51a2-43d1-9f12-345678901234".to_string(),
            title: "Test Post".to_string(),
            content: None,
            created_at: 1640995200,
            updated_at: 1640995200,
        };

        // Test serialization to JSON with null content
        let json = serde_json::to_string(&post).expect("Failed to serialize post");
        let expected = r#"{"id":"123e4567-e89b-12d3-a456-426614174000","user_id":"987fcdeb-51a2-43d1-9f12-345678901234","title":"Test Post","content":null,"created_at":1640995200,"updated_at":1640995200}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn test_post_deserialization() {
        let json = r#"{"id":"123e4567-e89b-12d3-a456-426614174000","user_id":"987fcdeb-51a2-43d1-9f12-345678901234","title":"Test Post","content":"This is test content","created_at":1640995200,"updated_at":1640995200}"#;
        
        // Test deserialization from JSON
        let post: Post = serde_json::from_str(json).expect("Failed to deserialize post");
        
        assert_eq!(post.id, "123e4567-e89b-12d3-a456-426614174000");
        assert_eq!(post.user_id, "987fcdeb-51a2-43d1-9f12-345678901234");
        assert_eq!(post.title, "Test Post");
        assert_eq!(post.content, Some("This is test content".to_string()));
        assert_eq!(post.created_at, 1640995200);
        assert_eq!(post.updated_at, 1640995200);
    }

    #[test]
    fn test_post_deserialization_without_content() {
        let json = r#"{"id":"123e4567-e89b-12d3-a456-426614174000","user_id":"987fcdeb-51a2-43d1-9f12-345678901234","title":"Test Post","content":null,"created_at":1640995200,"updated_at":1640995200}"#;
        
        // Test deserialization from JSON with null content
        let post: Post = serde_json::from_str(json).expect("Failed to deserialize post");
        
        assert_eq!(post.id, "123e4567-e89b-12d3-a456-426614174000");
        assert_eq!(post.user_id, "987fcdeb-51a2-43d1-9f12-345678901234");
        assert_eq!(post.title, "Test Post");
        assert_eq!(post.content, None);
        assert_eq!(post.created_at, 1640995200);
        assert_eq!(post.updated_at, 1640995200);
    }

    #[test]
    fn test_create_post_request_deserialization() {
        // Test with content
        let json_with_content = r#"{"user_id":"987fcdeb-51a2-43d1-9f12-345678901234","title":"Test Post","content":"Test content"}"#;
        let request: CreatePostRequest = serde_json::from_str(json_with_content).expect("Failed to deserialize CreatePostRequest");
        
        assert_eq!(request.user_id, "987fcdeb-51a2-43d1-9f12-345678901234");
        assert_eq!(request.title, "Test Post");
        assert_eq!(request.content, Some("Test content".to_string()));

        // Test without content
        let json_without_content = r#"{"user_id":"987fcdeb-51a2-43d1-9f12-345678901234","title":"Test Post"}"#;
        let request: CreatePostRequest = serde_json::from_str(json_without_content).expect("Failed to deserialize CreatePostRequest");
        
        assert_eq!(request.user_id, "987fcdeb-51a2-43d1-9f12-345678901234");
        assert_eq!(request.title, "Test Post");
        assert_eq!(request.content, None);

        // Test with null content
        let json_null_content = r#"{"user_id":"987fcdeb-51a2-43d1-9f12-345678901234","title":"Test Post","content":null}"#;
        let request: CreatePostRequest = serde_json::from_str(json_null_content).expect("Failed to deserialize CreatePostRequest");
        
        assert_eq!(request.user_id, "987fcdeb-51a2-43d1-9f12-345678901234");
        assert_eq!(request.title, "Test Post");
        assert_eq!(request.content, None);
    }
}