use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Entity not found: {0}")]
    NotFound(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Concurrent modification detected: {0}")]
    ConcurrentModification(String),
    
    #[error("Operation not permitted: {0}")]
    OperationNotPermitted(String),

    #[error("Unique constraint violation for {entity}.{field}: '{value}' already exists")]
    UniqueConstraintViolation {
        entity: String,
        field: String,
        value: String,
    },

    #[error("Foreign key constraint violation: {constraint_name}. Details: {details}")]
    ForeignKeyConstraintViolation {
        constraint_name: String,
        details: String,
    },

    #[error("Optimistic lock conflict for {entity} with id '{id}'. Expected version {expected_version}, found {actual_version}")]
    OptimisticLockConflict {
        entity: String,
        id: String,
        expected_version: i64,
        actual_version: i64,
    },

    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Database connection error: {0}")]
    ConnectionError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_error_variants_display() {
        let err = StorageError::NotFound("foo".to_string());
        assert_eq!(format!("{}", err), "Entity not found: foo");

        let err = StorageError::InvalidData("bad data".to_string());
        assert!(format!("{}", err).contains("bad data"));

        let err = StorageError::ConcurrentModification("row1".to_string());
        assert!(format!("{}", err).contains("Concurrent modification"));

        let err = StorageError::OperationNotPermitted("op".to_string());
        assert!(format!("{}", err).contains("Operation not permitted"));

        let err = StorageError::UniqueConstraintViolation {
            entity: "User".to_string(),
            field: "email".to_string(),
            value: "foo@bar.com".to_string(),
        };
        assert!(format!("{}", err).contains("User.email"));
        assert!(format!("{}", err).contains("foo@bar.com"));

        let err = StorageError::ForeignKeyConstraintViolation {
            constraint_name: "fk_user".to_string(),
            details: "user_id missing".to_string(),
        };
        assert!(format!("{}", err).contains("fk_user"));
        assert!(format!("{}", err).contains("user_id missing"));

        let err = StorageError::OptimisticLockConflict {
            entity: "Order".to_string(),
            id: "123".to_string(),
            expected_version: 2,
            actual_version: 1,
        };
        assert!(format!("{}", err).contains("Order"));
        assert!(format!("{}", err).contains("123"));
        assert!(format!("{}", err).contains("Expected version 2"));

        let err = StorageError::SerializationError("ser fail".to_string());
        assert!(format!("{}", err).contains("ser fail"));

        let err = StorageError::DeserializationError("de fail".to_string());
        assert!(format!("{}", err).contains("de fail"));

        let err = StorageError::ConnectionError("conn fail".to_string());
        assert!(format!("{}", err).contains("conn fail"));
    }
} 