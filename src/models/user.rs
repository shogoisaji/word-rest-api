use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 登録済みユーザーを表すドメインモデル。
/// `serde::{Serialize, Deserialize}` を derive しているので、そのまま JSON へシリアライズ可能。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// ユーザー作成 API が受け取るペイロード。
/// `Deserialize` のみ実装し、DB 保存時には `CreateUserRequest::into_user` で `User` に変換する。
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

/// ユーザー更新 API の入力。
/// 更新しないフィールドは `None` を渡すため、`Option<String>` として定義している。
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

impl User {
    /// UUID とタイムスタンプを自前で埋めた `User` を生成する。
    /// `Uuid::new_v4()` はランダム UUID、`Utc::now()` は現在時刻を取得するクロスプラットフォームな手段。
    pub fn new(name: String, email: String) -> Self {
        let now = Utc::now();
        
        User {
            id: Uuid::new_v4(),
            name,
            email,
            created_at: now,
            updated_at: now,
        }
    }

    /// 指定フィールドだけを書き換え、`updated_at` は常に最新にする。
    /// `if let Some` を使うことで、`match` よりも簡潔に Option を扱っている。
    pub fn update(&mut self, name: Option<String>, email: Option<String>) {
        if let Some(new_name) = name {
            self.name = new_name;
        }
        
        if let Some(new_email) = email {
            self.email = new_email;
        }
        
        self.updated_at = Utc::now();
    }
}

impl CreateUserRequest {
    /// ユーザー作成時のビジネスルール (空欄禁止・文字数上限・メール形式) を検証する。
    /// 失敗時は `Err(String)` を返し、API 層で `ApiError::Validation` に変換される。
    pub fn validate(&self) -> Result<(), String> {
        // Validate name
        if self.name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        
        if self.name.len() > 100 {
            return Err("Name cannot exceed 100 characters".to_string());
        }

        // Validate email
        if self.email.trim().is_empty() {
            return Err("Email cannot be empty".to_string());
        }
        
        if !is_valid_email(&self.email) {
            return Err("Invalid email format".to_string());
        }
        
        if self.email.len() > 255 {
            return Err("Email cannot exceed 255 characters".to_string());
        }

        Ok(())
    }

    /// 受け取った入力をトリム・小文字化して `User` に変換する。
    /// フィールドをクリーンアップする責務をこの層に閉じ込めることで、DB 層の複雑さを減らしている。
    pub fn into_user(self) -> User {
        User::new(self.name.trim().to_string(), self.email.trim().to_lowercase())
    }
}

impl UpdateUserRequest {
    /// 更新時は少なくともどちらか 1 フィールドが必要、というルールを表現する。
    /// `Option` の中身が存在するときのみ、`trim` や長さチェックをかけている。
    pub fn validate(&self) -> Result<(), String> {
        // Check if at least one field is provided
        if self.name.is_none() && self.email.is_none() {
            return Err("At least one field (name or email) must be provided for update".to_string());
        }

        // Validate name if provided
        if let Some(ref name) = self.name {
            if name.trim().is_empty() {
                return Err("Name cannot be empty".to_string());
            }
            
            if name.len() > 100 {
                return Err("Name cannot exceed 100 characters".to_string());
            }
        }

        // Validate email if provided
        if let Some(ref email) = self.email {
            if email.trim().is_empty() {
                return Err("Email cannot be empty".to_string());
            }
            
            if !is_valid_email(email) {
                return Err("Invalid email format".to_string());
            }
            
            if email.len() > 255 {
                return Err("Email cannot exceed 255 characters".to_string());
            }
        }

        Ok(())
    }

    /// 名前をトリムし、空なら `None` にするユーティリティ。
    /// 返り値も `Option<String>` なので、そのまま SQL の動的組み立てに流用できる。
    pub fn get_normalized_name(&self) -> Option<String> {
        self.name.as_ref().map(|n| n.trim().to_string())
    }

    /// メールアドレスをトリムして小文字化する。
    /// メールは大小区別しないことが多いため、ここで正規化しておくと照合漏れを防げる。
    pub fn get_normalized_email(&self) -> Option<String> {
        self.email.as_ref().map(|e| e.trim().to_lowercase())
    }
}

/// シンプルなメールフォーマット検証。
/// 正規表現を使わず、`split('@')` などで最小限のルールをチェックしている。
fn is_valid_email(email: &str) -> bool {
    // Basic email validation - contains @ and has parts before and after
    let parts: Vec<&str> = email.split('@').collect();
    
    if parts.len() != 2 {
        return false;
    }
    
    let local = parts[0];
    let domain = parts[1];
    
    // Check local part
    if local.is_empty() || local.len() > 64 {
        return false;
    }
    
    // Check domain part
    if domain.is_empty() || domain.len() > 253 {
        return false;
    }
    
    // Domain should contain at least one dot
    if !domain.contains('.') {
        return false;
    }
    
    // Basic character validation
    let valid_chars = |c: char| c.is_alphanumeric() || ".-_+".contains(c);
    
    local.chars().all(valid_chars) && domain.chars().all(|c| c.is_alphanumeric() || ".-".contains(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new("John Doe".to_string(), "john@example.com".to_string());
        
        assert_ne!(user.id, Uuid::nil());
        assert_eq!(user.name, "John Doe");
        assert_eq!(user.email, "john@example.com");
        assert!(user.created_at <= Utc::now());
        assert_eq!(user.created_at, user.updated_at);
    }

    #[test]
    fn test_user_update() {
        let mut user = User::new("John Doe".to_string(), "john@example.com".to_string());
        let original_created_at = user.created_at;
        let original_updated_at = user.updated_at;
        
        // Sleep for 1 millisecond to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        user.update(Some("Jane Doe".to_string()), None);
        
        assert_eq!(user.name, "Jane Doe");
        assert_eq!(user.email, "john@example.com");
        assert_eq!(user.created_at, original_created_at);
        assert!(user.updated_at > original_updated_at);
    }

    #[test]
    fn test_create_user_request_validation() {
        // Valid request
        let valid_request = CreateUserRequest {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        // Empty name
        let invalid_name = CreateUserRequest {
            name: "".to_string(),
            email: "john@example.com".to_string(),
        };
        assert!(invalid_name.validate().is_err());

        // Invalid email
        let invalid_email = CreateUserRequest {
            name: "John Doe".to_string(),
            email: "invalid-email".to_string(),
        };
        assert!(invalid_email.validate().is_err());
    }

    #[test]
    fn test_update_user_request_validation() {
        // Valid update with name
        let valid_update = UpdateUserRequest {
            name: Some("Jane Doe".to_string()),
            email: None,
        };
        assert!(valid_update.validate().is_ok());

        // Empty update
        let empty_update = UpdateUserRequest {
            name: None,
            email: None,
        };
        assert!(empty_update.validate().is_err());

        // Invalid email in update
        let invalid_email_update = UpdateUserRequest {
            name: None,
            email: Some("invalid-email".to_string()),
        };
        assert!(invalid_email_update.validate().is_err());
    }

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name@domain.co.uk"));
        assert!(is_valid_email("user+tag@example.org"));
        
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email("user@domain"));
        assert!(!is_valid_email(""));
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            id: Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            created_at: DateTime::parse_from_rfc3339("2022-01-01T00:00:00Z").unwrap().with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339("2022-01-01T00:00:00Z").unwrap().with_timezone(&Utc),
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&user).expect("Failed to serialize user");
        let expected = r#"{"id":"123e4567-e89b-12d3-a456-426614174000","name":"John Doe","email":"john@example.com","created_at":"2022-01-01T00:00:00Z","updated_at":"2022-01-01T00:00:00Z"}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn test_user_deserialization() {
        let json = r#"{"id":"123e4567-e89b-12d3-a456-426614174000","name":"John Doe","email":"john@example.com","created_at":"2022-01-01T00:00:00Z","updated_at":"2022-01-01T00:00:00Z"}"#;
        
        // Test deserialization from JSON
        let user: User = serde_json::from_str(json).expect("Failed to deserialize user");
        
        assert_eq!(user.id, Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap());
        assert_eq!(user.name, "John Doe");
        assert_eq!(user.email, "john@example.com");
        assert_eq!(user.created_at, DateTime::parse_from_rfc3339("2022-01-01T00:00:00Z").unwrap().with_timezone(&Utc));
        assert_eq!(user.updated_at, DateTime::parse_from_rfc3339("2022-01-01T00:00:00Z").unwrap().with_timezone(&Utc));
    }

    #[test]
    fn test_create_user_request_deserialization() {
        let json = r#"{"name":"Jane Doe","email":"jane@example.com"}"#;
        
        let request: CreateUserRequest = serde_json::from_str(json).expect("Failed to deserialize CreateUserRequest");
        
        assert_eq!(request.name, "Jane Doe");
        assert_eq!(request.email, "jane@example.com");
    }

    #[test]
    fn test_update_user_request_deserialization() {
        // Test with both fields
        let json_both = r#"{"name":"Updated Name","email":"updated@example.com"}"#;
        let request: UpdateUserRequest = serde_json::from_str(json_both).expect("Failed to deserialize UpdateUserRequest");
        assert_eq!(request.name, Some("Updated Name".to_string()));
        assert_eq!(request.email, Some("updated@example.com".to_string()));

        // Test with only name
        let json_name_only = r#"{"name":"Updated Name"}"#;
        let request: UpdateUserRequest = serde_json::from_str(json_name_only).expect("Failed to deserialize UpdateUserRequest");
        assert_eq!(request.name, Some("Updated Name".to_string()));
        assert_eq!(request.email, None);

        // Test with only email
        let json_email_only = r#"{"email":"updated@example.com"}"#;
        let request: UpdateUserRequest = serde_json::from_str(json_email_only).expect("Failed to deserialize UpdateUserRequest");
        assert_eq!(request.name, None);
        assert_eq!(request.email, Some("updated@example.com".to_string()));
    }
}
