//! Shared input validators used across calculation methods.
//!
//! These cover the cross-cutting rules in `04-rust-engine-standards.md`
//! ("Input Validation"). Method-specific validation still belongs in each
//! method module; put a check here only once it is needed by more than
//! one method.

use crate::error::{Error, Result};

pub fn validate_alpha(alpha: f64) -> Result<()> {
    if alpha > 0.0 && alpha < 1.0 {
        Ok(())
    } else {
        Err(Error::InvalidInput {
            field: "alpha".into(),
            message: "must be greater than 0 and less than 1".into(),
        })
    }
}

pub fn validate_power(power: f64) -> Result<()> {
    if power > 0.0 && power < 1.0 {
        Ok(())
    } else {
        Err(Error::InvalidInput {
            field: "power".into(),
            message: "must be greater than 0 and less than 1".into(),
        })
    }
}

pub fn validate_positive(field: &str, value: f64) -> Result<()> {
    if value > 0.0 {
        Ok(())
    } else {
        Err(Error::InvalidInput {
            field: field.into(),
            message: "must be positive".into(),
        })
    }
}

pub fn validate_correlation(correlation: f64) -> Result<()> {
    if correlation > -1.0 && correlation < 1.0 {
        Ok(())
    } else {
        Err(Error::InvalidInput {
            field: "baseline_outcome_correlation".into(),
            message: "must be greater than -1 and less than 1".into(),
        })
    }
}

pub fn validate_dropout_rate(dropout_rate: f64) -> Result<()> {
    if (0.0..1.0).contains(&dropout_rate) {
        Ok(())
    } else {
        Err(Error::InvalidInput {
            field: "dropout_rate".into(),
            message: "must be at least 0 and less than 1".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_rejects_boundary_and_outside_values() {
        assert!(validate_alpha(0.05).is_ok());
        assert!(validate_alpha(0.0).is_err());
        assert!(validate_alpha(1.0).is_err());
        assert!(validate_alpha(-0.1).is_err());
    }

    #[test]
    fn power_rejects_boundary_and_outside_values() {
        assert!(validate_power(0.8).is_ok());
        assert!(validate_power(0.0).is_err());
        assert!(validate_power(1.0).is_err());
    }

    #[test]
    fn positive_rejects_zero_and_negative() {
        assert!(validate_positive("standard_deviation", 1.0).is_ok());
        assert!(validate_positive("standard_deviation", 0.0).is_err());
        assert!(validate_positive("standard_deviation", -1.0).is_err());
    }

    #[test]
    fn correlation_rejects_boundary_values() {
        assert!(validate_correlation(0.5).is_ok());
        assert!(validate_correlation(-0.99).is_ok());
        assert!(validate_correlation(0.99).is_ok());
        assert!(validate_correlation(-1.0).is_err());
        assert!(validate_correlation(1.0).is_err());
    }

    #[test]
    fn dropout_rate_allows_zero_rejects_one() {
        assert!(validate_dropout_rate(0.0).is_ok());
        assert!(validate_dropout_rate(0.2).is_ok());
        assert!(validate_dropout_rate(1.0).is_err());
        assert!(validate_dropout_rate(-0.01).is_err());
    }
}
