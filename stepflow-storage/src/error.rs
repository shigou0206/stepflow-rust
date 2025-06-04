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