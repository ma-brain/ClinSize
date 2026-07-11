//! Calculation methods, one submodule per endpoint category.
//!
//! Each method follows the shape documented in `02-architecture.md`
//! ("Core Crate Design"): a typed input struct, a typed output struct,
//! a validation function, a calculation function, unit tests, and
//! reference examples. See `templates/new-calculation-method.md` before
//! adding a method here.

pub mod binary;
pub mod continuous;
pub mod survival;
