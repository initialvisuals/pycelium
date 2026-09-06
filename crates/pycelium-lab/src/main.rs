mod render;

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use pycelium_core::{SimConfig, World};

use crate::render::{render_ascii, write_occupancy_pgm};

/// Pycelium PC sim-lab harness: run measurable growth experiments.
#[derive(Parser, Debug)]
#[command(name = "pycelium-lab", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run N steps and print data-science metrics.
    Bench(RunArgs),
    /// Run N steps and print a downsampled ASCII slice of the grid.
    View(RunArgs),
}

#[derive(clap::Args, Debug)]
struct RunArgs {
    /// Grid width in cells. Increase this (with --height) to soak RAM.
    #[arg(long, default_value_t = 512)]
    width: u32,
    /// Grid height in cells.
    #[arg(long, default_value_t = 512)]
    height: u32,
    /// Simulation steps to run.
    #[arg(long, default_value_t = 200)]
    steps: u32,
    /// RNG seed for a repeatable run.
    #[arg(long, default_value_t = 1)]
    seed: u64,
    /// Inoculum / active tip count at t=0.
    #[arg(long, default_value_t = 4)]
    tips: u32,
    /// Scattered nutrient patches.
    #[arg(long, default_value_t = 48)]
    food: u32,
    /// Nutrient patch radius in cells.
    #[arg(long, default_value_t = 12)]
    radius: u32,
    /// Print a JSON object instead of the text report.
    #[arg(long)]
    json: bool,
    /// Also print an ASCII slice after the run (bench only; view always does).
    #[arg(long)]
    ascii: bool,
    /// ASCII slice width in characters.
    #[arg(long, default_value_t = 80)]
    view_width: u32,
    /// ASCII slice height in rows.
    #[arg(long, default_value_t = 24)]
    view_height: u32,
    /// Write occupancy as a binary PGM (viewable in any image tool).
    #[arg(long)]
    export_pgm: Option<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let (args, force_ascii) = match cli.command {
        Command::Bench(args) => (args, false),
        Command::View(args) => (args, true),
    };

    if let Err(err) = run(args, force_ascii) {
        eprintln!("error: {err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run(args: RunArgs, force_ascii: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config = SimConfig {
        width: args.width,
        height: args.height,
        seed: args.seed,
        initial_tips: args.tips,
        nutrient_patches: args.food,
        patch_radius: args.radius,
        ..SimConfig::default()
    };

    let mut world = World::new(config)?;
    let report = world.run_experiment(args.steps);

    let mut out = io::stdout().lock();
    if args.json {
        writeln!(out, "{}", report.render_json())?;
    } else {
        write!(out, "{}", report.render_text())?;
    }

    if force_ascii || args.ascii {
        writeln!(out)?;
        write!(
            out,
            "{}",
            render_ascii(
                world.occupancy(),
                world.nutrient(),
                args.view_width.max(8),
                args.view_height.max(4),
            )
        )?;
    }

    if let Some(path) = args.export_pgm {
        let bytes = write_occupancy_pgm(world.occupancy());
        fs::write(&path, bytes)?;
        if !args.json {
            writeln!(out, "wrote occupancy PGM: {}", path.display())?;
        }
    }

    Ok(())
}
