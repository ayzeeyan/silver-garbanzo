//! Error types for the decompilation pipeline.
//!
//! Provides granular errors for each stage of the pipeline and a top-level
//! `DeobfuscationError` that wraps them.

use thiserror::Error;

/// The top-level error type representing a failure anywhere in the pipeline.
#[derive(Debug, Error)]
pub enum DeobfuscationError {
    /// Failed to parse the input file (bytecode or source).
    #[error("Parsing failed: {0}")]
    Parse(#[from] ParserError),

    /// Failed during IR construction or optimization.
    #[error("IR manipulation failed: {0}")]
    Ir(#[from] IrError),

    /// Failed during emulation/string resolution.
    #[error("Emulation failed: {0}")]
    Emu(#[from] EmuError),

    /// Failed to generate valid Lua source code.
    #[error("Code generation failed: {0}")]
    CodeGen(#[from] CodeGenError),

    /// Failed during obfuscator profile detection or transformation.
    #[error("Profile processing failed: {0}")]
    Profile(#[from] ProfileError),
}

/// Errors occurring during the parsing stage.
#[derive(Debug, Error)]
pub enum ParserError {
    /// The input format could not be identified as valid Lua bytecode or known obfuscated source.
    #[error("Unrecognized file format")]
    UnrecognizedFormat,

    /// The binary chunk header is invalid or corrupted.
    #[error("Corrupted header: bytes {bytes:?} (attempted: {attempted_recoveries:?})")]
    CorruptedHeader {
        /// The first 12 bytes of the chunk, represented as a hex string.
        bytes: String,
        /// The recovery strategies attempted before failing.
        attempted_recoveries: Vec<&'static str>,
    },

    /// The binary chunk contains invalid opcodes or structures.
    #[error("Malformed chunk: {0}")]
    MalformedChunk(String),
}

/// Errors occurring during IR manipulation (lifting, SSA, optimization).
#[derive(Debug, Error)]
pub enum IrError {
    /// An operation attempted to use an invalid block ID.
    #[error("Invalid block ID: {0}")]
    InvalidBlockId(u32),

    /// An operation attempted to use an invalid function ID.
    #[error("Invalid function ID: {0}")]
    InvalidFunctionId(u32),

    /// An internal invariant was violated during optimization passes.
    #[error("Optimization pass failed: {0}")]
    OptimizationFailure(String),

    /// Generic IR validation error.
    #[error("Malformed IR: {0}")]
    Malformed(String),
}

/// Errors occurring within the bounded emulator execution.
#[derive(Debug, Error)]
pub enum EmuError {
    /// A resource limit (steps or memory) was exceeded during emulation.
    #[error("Resource limit exceeded: {kind} limit was {limit}")]
    ResourceLimitExceeded {
        /// The type of limit (e.g., "instructions" or "memory").
        kind: &'static str,
        /// The limit threshold that was breached.
        limit: usize,
    },

    /// The emulator encountered an opcode it does not support.
    #[error("Unsupported opcode encountered: {0}")]
    UnsupportedOpcode(String),

    /// The emulator attempted an invalid operation (e.g., arithmetic on a table).
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

/// Errors occurring during the code generation stage.
#[derive(Debug, Error)]
pub enum CodeGenError {
    /// A required variable could not be resolved or renamed.
    #[error("Missing variable binding: {0}")]
    MissingBinding(String),

    /// The IR contains an un-handled or structurally invalid construct for emission.
    #[error("Emission failed: {0}")]
    EmissionFailure(String),
}

/// Errors occurring within a specific obfuscator profile.
#[derive(Debug, Error)]
pub enum ProfileError {
    /// The profile encountered an expected structure that was missing or invalid.
    #[error("Pattern matching failed: {0}")]
    PatternMismatch(String),

    /// The profile failed to apply an IR transformation safely.
    #[error("Transformation failed: {0}")]
    TransformationFailure(String),
}
