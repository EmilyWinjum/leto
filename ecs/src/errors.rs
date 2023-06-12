use core::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum EcsError {
    Placeholder(String),
}

impl fmt::Display for EcsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Self::Placeholder(message) => message,
        })
    }
}

impl Error for EcsError {}

impl From<EntityError> for EcsError {
    fn from(err: EntityError) -> Self {
        Self::Placeholder(err.to_string())
    }
}

#[derive(Debug)]
pub enum StoreError {
    CannotCastToType,
    TypeNotFound,
    StorageNotFound,
    Placeholder,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Self::CannotCastToType => "cannot cast to specified type",
            Self::TypeNotFound => "the target type could not be found",
            Self::StorageNotFound => "storage not contained in archetype",
            Self::Placeholder => "placeholder",
        })
    }
}

impl Error for StoreError {}

#[derive(Debug)]
pub enum EntityError {
    TooManyEntities,
    FreedListTooSmall,
    NotFound,
    WrongGen,
    AlreadyFreed,
}

impl fmt::Display for EntityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Self::TooManyEntities => "too many entities",
            Self::FreedListTooSmall => "freed list is too small for request",
            Self::NotFound => "entity not found",
            Self::WrongGen => "generations don't match",
            Self::AlreadyFreed => "entity already freed",
        })
    }
}

impl Error for EntityError {}
