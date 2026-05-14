use crate::error::DeobfuscationError;
use crate::ir_engine::IrModule;
use crate::lua_parser::LuaChunk;

/// Context provided to profile detectors during triaging.
#[allow(missing_docs)]
pub struct DetectionContext<'m> {
    pub module: &'m IrModule,
    pub raw_source: Option<&'m str>,
    pub chunk: Option<&'m LuaChunk<'m>>,
}

/// Interface that every obfuscator signature plugin implements.
#[allow(missing_docs)]
pub trait ObfuscatorProfile: Send + Sync {
    /// Human-readable name (e.g., "Ironbrew 2").
    fn name(&self) -> &'static str;

    /// Return a confidence score [0.0, 1.0].
    fn detect(&self, ctx: &DetectionContext<'_>) -> f64;

    /// Mutate IR to undo specific patterns BEFORE code generation.
    fn pre_decompile_pass(&self, ir: &mut IrModule) -> Result<(), DeobfuscationError>;

    /// Post-process emitted Lua source string.
    fn post_decompile_pass(&self, source: &mut String) -> Result<(), DeobfuscationError>;
}

/// Global registry mapping specific signatures to profile instances.
#[derive(Default)]
#[allow(missing_docs)]
pub struct ProfileRegistry {
    profiles: Vec<Box<dyn ObfuscatorProfile>>,
}

impl ProfileRegistry {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        let mut registry = Self::default();
        // Lowest priority generic fallback
        registry.register(Box::new(super::generic::GenericProfile));
        registry.register(Box::new(super::ironbrew::IronbrewProfile));
        registry.register(Box::new(super::moonsec::MoonSecProfile));
        registry.register(Box::new(super::luraph::LuraphProfile));
        registry
    }

    #[allow(missing_docs)]
    pub fn register(&mut self, profile: Box<dyn ObfuscatorProfile>) {
        self.profiles.push(profile);
    }

    #[allow(missing_docs)]
    pub fn detect_all(&self, ctx: &DetectionContext<'_>) -> Vec<(&dyn ObfuscatorProfile, f64)> {
        let mut results: Vec<_> = self.profiles
            .iter()
            .map(|p| (p.as_ref(), p.detect(ctx)))
            .collect();

        // Sort descending by score
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    #[allow(missing_docs)]
    pub fn get_by_name(&self, name: &str) -> Option<&dyn ObfuscatorProfile> {
        self.profiles.iter().find(|p| p.name().eq_ignore_ascii_case(name)).map(|p| p.as_ref())
    }
}
