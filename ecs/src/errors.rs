use core::fmt;
use std::{error::Error, num::TryFromIntError};

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

impl From<ArchetypeError> for EcsError {
    fn from(err: ArchetypeError) -> Self {
        Self::Placeholder(err.to_string())
    }
}


#[derive(Debug)]
pub enum StoreError {
    CannotCastToType,
    TypeNotInBundle,
    Placeholder,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Self::CannotCastToType => "cannot cast to specified type",
            Self::TypeNotInBundle => "type is not present in bundle",
            Self::Placeholder => "placeholder",
        })
    }
}

impl Error for StoreError {}


#[derive(Debug)]
pub enum EntityError {
    TooManyEntities(String),
    FreedListTooSmall,
    NotFound,
    WrongGen,
    AlreadyFreed,
}

impl fmt::Display for EntityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Self::TooManyEntities(message) => 
                format!("too many entities{}", message),
            Self::FreedListTooSmall => 
                format!("freed list is too small for request"),
            Self::NotFound => 
                format!("entity not found"),
            Self::WrongGen => 
                format!("generations don't match"),
            Self::AlreadyFreed => 
                format!("entity already freed"),
        }.as_str())
    }
}

impl Error for EntityError {}

impl From<TryFromIntError> for EntityError {
    fn from(err: TryFromIntError) -> Self {
        Self::TooManyEntities(format!(": ({})", err))
    }
}


#[derive(Debug)]
pub enum ArchetypeError {
    TypeNotAvailable,
    NoEdge,
    Placeholder,
}

impl fmt::Display for ArchetypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Self::TypeNotAvailable => "type not available in archetype",
            Self::NoEdge => "no archetype edge exists for type",
            Self::Placeholder => "placeholder",
        })
    }
}

impl Error for ArchetypeError {}