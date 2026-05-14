use anyhow::{Context, Result};
use clap::Parser;
use log::{info, trace};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use lunadec::ir_engine::{lift_to_ir, opt::optimize_module};
use lunadec::lua_parser::{parse_chunk, parse_source};
use lunadec::profiles::{DetectionContext, ProfileRegistry};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(required = true)]
    input: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(short, long)]
    profile: Option<String>,

    #[arg(long, default_value_t = 0.5)]
    profile_threshold: f64,

    #[arg(short = 'v', action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(long)]
    no_color: bool,

    #[arg(long)]
    dump_ir: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let log_level = match args.verbose {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    let mut builder = env_logger::Builder::new();
    builder.filter_level(log_level);
    if args.no_color {
        builder.format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()));
    } else {
        builder.format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()));
    }
    builder.init();

    info!("Starting triage...");

    let file_bytes = fs::read(&args.input).context("Failed to read input file")?;

    let is_bytecode = file_bytes.len() >= 4 && &file_bytes[..4] == b"\x1bLua";
    let is_mangled = file_bytes.len() >= 4 && &file_bytes[1..4] == b"Lua";

    let chunk = if is_bytecode || is_mangled {
        info!("File identified as KIND_BYTECODE");
        parse_chunk(&file_bytes).context("Failed to parse bytecode chunk")?
    } else {
        info!("File identified as KIND_OBFSOURCE");
        let src_str = std::str::from_utf8(&file_bytes).context("File is not valid UTF-8 source")?;
        parse_source(src_str).context("Failed to reconstruct chunk from obfuscated source")?
    };

    info!("Parsed chunk (FunctionProto roots: 1)");

    trace!("Lifting IR");
    let mut ir_module = lift_to_ir(&chunk).context("Failed to lift to IR")?;

    let block_count = ir_module.functions.iter().map(|f| f.blocks.len()).sum::<usize>();
    info!("SSA lifted, blocks: {}", block_count);

    trace!("Running optimization pipeline");
    let passes = optimize_module(&mut ir_module).context("Failed optimization pass")?;
    info!("Optimizations reached fixed point ({} passes)", passes);

    let mut raw_source_str = None;
    if !is_bytecode && !is_mangled {
        raw_source_str = std::str::from_utf8(&file_bytes).ok();
    }

    let detect_ctx = DetectionContext {
        module: &ir_module,
        raw_source: raw_source_str,
        chunk: Some(&chunk),
    };

    let registry = ProfileRegistry::new();

    let profile = if let Some(p_name) = args.profile {
        registry.get_by_name(&p_name).context(format!("Profile '{}' not found", p_name))?
    } else {
        let sorted = registry.detect_all(&detect_ctx);
        if sorted.is_empty() || sorted[0].1 < args.profile_threshold {
            registry.get_by_name("Generic").expect("Invariant violation")
        } else {
            sorted[0].0
        }
    };

    let p_score = profile.detect(&detect_ctx);
    info!("Detected profile: '{}' (score {})", profile.name(), p_score);

    trace!("Applying pre-decompile transformations");
    profile.pre_decompile_pass(&mut ir_module).context("Pre-decompile transform failed")?;
    info!("Deobfuscation pass successful");

    trace!("Running emulation layer string resolution");
    let decrypted = lunadec::emu_layer::resolve_strings(&mut ir_module).context("Emulation step failed")?;
    info!("Emulation resolved {} encrypted strings", decrypted);

    info!("Generating Lua 5.1 source");
    let mut final_source = lunadec::code_gen::generate(&ir_module).context("Code generation failed")?;

    profile.post_decompile_pass(&mut final_source).context("Post-decompile cleanup failed")?;

    if let Some(out_path) = args.output {
        fs::write(&out_path, &final_source).context("Failed to write output source")?;
        info!("Output written to {:?}", out_path);
    } else {
        println!("{}", final_source);
    }

    Ok(())
}
