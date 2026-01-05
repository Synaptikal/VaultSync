//! Input Validation Module
//!
//! Provides validation utilities for API inputs. All validation errors
//! should be caught BEFORE data reaches business logic.

use crate::errors::VaultSyncError;

/// Validation result type
pub type ValidationResult<T> = std::result::Result<T, ValidationError>;

/// Validation error with field-level details
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl From<ValidationError> for VaultSyncError {
    fn from(e: ValidationError) -> Self {
        VaultSyncError::ValidationError(e.to_string())
    }
}

/// Trait for validatable types
pub trait Validate {
    fn validate(&self) -> ValidationResult<()>;
}

/// Common validation functions
pub mod rules {
    use super::*;

    /// Ensure string is not empty or whitespace-only
    pub fn not_empty(field: &str, value: &str) -> ValidationResult<()> {
        if value.trim().is_empty() {
            Err(ValidationError {
                field: field.to_string(),
                message: "must not be empty".to_string(),
            })
        } else {
            Ok(())
        }
    }

    /// Ensure string length is within bounds
    pub fn length_range(field: &str, value: &str, min: usize, max: usize) -> ValidationResult<()> {
        let len = value.len();
        if len < min {
            Err(ValidationError {
                field: field.to_string(),
                message: format!("must be at least {} characters", min),
            })
        } else if len > max {
            Err(ValidationError {
                field: field.to_string(),
                message: format!("must be at most {} characters", max),
            })
        } else {
            Ok(())
        }
    }

    /// Ensure numeric value is positive
    pub fn positive<T: PartialOrd + Default + std::fmt::Display>(
        field: &str,
        value: T,
    ) -> ValidationResult<()> {
        if value <= T::default() {
            Err(ValidationError {
                field: field.to_string(),
                message: "must be positive".to_string(),
            })
        } else {
            Ok(())
        }
    }

    /// Ensure numeric value is non-negative
    pub fn non_negative<T: PartialOrd + Default>(field: &str, value: T) -> ValidationResult<()> {
        if value < T::default() {
            Err(ValidationError {
                field: field.to_string(),
                message: "must not be negative".to_string(),
            })
        } else {
            Ok(())
        }
    }

    /// Ensure numeric value is within range
    pub fn in_range<T: PartialOrd + std::fmt::Display>(
        field: &str,
        value: T,
        min: T,
        max: T,
    ) -> ValidationResult<()> {
        if value < min {
            Err(ValidationError {
                field: field.to_string(),
                message: format!("must be at least {}", min),
            })
        } else if value > max {
            Err(ValidationError {
                field: field.to_string(),
                message: format!("must be at most {}", max),
            })
        } else {
            Ok(())
        }
    }

    /// Validate email format (basic)
    pub fn email_format(field: &str, value: &str) -> ValidationResult<()> {
        if value.is_empty() {
            return Ok(()); // Empty is allowed (use not_empty if required)
        }

        if !value.contains('@') || !value.contains('.') {
            Err(ValidationError {
                field: field.to_string(),
                message: "must be a valid email address".to_string(),
            })
        } else {
            Ok(())
        }
    }

    /// Validate year is reasonable
    pub fn valid_year(field: &str, year: i32) -> ValidationResult<()> {
        if year < 1900 || year > 2100 {
            Err(ValidationError {
                field: field.to_string(),
                message: "must be a valid year between 1900 and 2100".to_string(),
            })
        } else {
            Ok(())
        }
    }
}

/// Validator builder for complex validations
pub struct Validator {
    errors: Vec<ValidationError>,
}

impl Validator {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn check(mut self, result: ValidationResult<()>) -> Self {
        if let Err(e) = result {
            self.errors.push(e);
        }
        self
    }

    pub fn finish(self) -> ValidationResult<()> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            // Return first error (could be extended to return all)
            // SECURITY FIX: Safe to expect - we just checked is_empty() above
            Err(self.errors.into_iter().next().expect("errors not empty"))
        }
    }

    pub fn finish_all(self) -> Result<(), Vec<ValidationError>> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_empty() {
        assert!(rules::not_empty("name", "John").is_ok());
        assert!(rules::not_empty("name", "").is_err());
        assert!(rules::not_empty("name", "   ").is_err());
    }

    #[test]
    fn test_length_range() {
        assert!(rules::length_range("name", "John", 1, 100).is_ok());
        assert!(rules::length_range("name", "", 1, 100).is_err());
        assert!(rules::length_range("name", "x".repeat(101).as_str(), 1, 100).is_err());
    }

    #[test]
    fn test_positive() {
        assert!(rules::positive("quantity", 5).is_ok());
        assert!(rules::positive("quantity", 0).is_err());
        assert!(rules::positive("quantity", -1).is_err());
    }

    #[test]
    fn test_validator_chaining() {
        let result = Validator::new()
            .check(rules::not_empty("name", "John"))
            .check(rules::positive("age", 25))
            .finish();

        assert!(result.is_ok());

        let result = Validator::new()
            .check(rules::not_empty("name", ""))
            .check(rules::positive("age", -1))
            .finish_all();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().len(), 2);
    }

    #[test]
    fn test_email_format() {
        assert!(rules::email_format("email", "test@example.com").is_ok());
        assert!(rules::email_format("email", "").is_ok()); // Empty allowed
        assert!(rules::email_format("email", "invalid").is_err());
        assert!(rules::email_format("email", "no-at-sign.com").is_err());
    }
}
