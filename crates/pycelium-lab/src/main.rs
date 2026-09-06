mod args;
mod render;

use std::fs;
use std::io::{self, Write};
use std::process::ExitCode;

use pycelium_core::{SimConfig, World};

use crate::args::{parse_args, Command};
use crate::render::{render_ascii, write_occupancy_pgm};

fn main() -> ExitCode {
    let (args, force_ascii) = match parse_args(std::env::args().skip(1)) {
        Ok(Command::Help(text)) => {
            print!("{text}");
            return ExitCode::SUCCESS;
        }
        Ok(Command::Bench(args)) => (args, false),
        Ok(Command::View(args)) => (args, true),
        Err(err) => {
            eprintln!("error: {err}");
            eprintln!("try: pycelium-lab help");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = run(args, force_ascii) {
        eprintln!("error: {err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run(args: args::RunArgs, force_ascii: bool) -> Result<(), Box<dyn std::error::Error>> {
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
