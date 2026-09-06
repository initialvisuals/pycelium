use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RunArgs {
    pub width: u32,
    pub height: u32,
    pub steps: u32,
    pub seed: u64,
    pub tips: u32,
    pub food: u32,
    pub radius: u32,
    pub json: bool,
    pub ascii: bool,
    pub view_width: u32,
    pub view_height: u32,
    pub export_pgm: Option<PathBuf>,
}

impl Default for RunArgs {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            steps: 200,
            seed: 1,
            tips: 4,
            food: 48,
            radius: 12,
            json: false,
            ascii: false,
            view_width: 80,
            view_height: 24,
            export_pgm: None,
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Bench(RunArgs),
    View(RunArgs),
    Help(String),
}

pub fn usage() -> String {
    "Pycelium PC sim-lab harness: run measurable growth experiments.\n\
     \n\
     Usage:\n\
       pycelium-lab bench [options]\n\
       pycelium-lab view  [options]\n\
       pycelium-lab help\n\
     \n\
     Options:\n\
       --width N          Grid width in cells (default 512). Bump with --height to soak RAM.\n\
       --height N         Grid height in cells (default 512)\n\
       --steps N          Simulation steps (default 200)\n\
       --seed N           RNG seed (default 1)\n\
       --tips N           Inoculum / active tips at t=0 (default 4)\n\
       --food N           Nutrient patches (default 48)\n\
       --radius N         Nutrient patch radius (default 12)\n\
       --json             Print metrics as JSON\n\
       --ascii            Also print an ASCII slice (view always does)\n\
       --view-width N     ASCII slice width (default 80)\n\
       --view-height N    ASCII slice height (default 24)\n\
       --export-pgm PATH  Write occupancy as a binary PGM\n"
        .to_string()
}

pub fn parse_args<I, S>(args: I) -> Result<Command, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut iter = args.into_iter();
    let cmd = match iter.next() {
        None => return Ok(Command::Help(usage())),
        Some(s) => s.as_ref().to_string(),
    };

    match cmd.as_str() {
        "help" | "-h" | "--help" => Ok(Command::Help(usage())),
        "bench" => Ok(Command::Bench(parse_run_args(iter)?)),
        "view" => Ok(Command::View(parse_run_args(iter)?)),
        other => Err(format!("unknown command '{other}'")),
    }
}

fn parse_run_args<I, S>(iter: I) -> Result<RunArgs, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut out = RunArgs::default();
    let mut args = iter.into_iter();
    while let Some(raw) = args.next() {
        let flag = raw.as_ref();
        match flag {
            "--json" => out.json = true,
            "--ascii" => out.ascii = true,
            "--width" => out.width = next_u32(&mut args, flag)?,
            "--height" => out.height = next_u32(&mut args, flag)?,
            "--steps" => out.steps = next_u32(&mut args, flag)?,
            "--seed" => out.seed = next_u64(&mut args, flag)?,
            "--tips" => out.tips = next_u32(&mut args, flag)?,
            "--food" => out.food = next_u32(&mut args, flag)?,
            "--radius" => out.radius = next_u32(&mut args, flag)?,
            "--view-width" => out.view_width = next_u32(&mut args, flag)?,
            "--view-height" => out.view_height = next_u32(&mut args, flag)?,
            "--export-pgm" => {
                let path = next_value(&mut args, flag)?;
                out.export_pgm = Some(PathBuf::from(path));
            }
            "-h" | "--help" => return Err("help requested".into()),
            other => return Err(format!("unknown option '{other}'")),
        }
    }
    Ok(out)
}

fn next_value<I, S>(iter: &mut I, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    iter.next()
        .map(|s| s.as_ref().to_string())
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn next_u32<I, S>(iter: &mut I, flag: &str) -> Result<u32, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let value = next_value(iter, flag)?;
    value
        .parse::<u32>()
        .map_err(|_| format!("{flag} expected a u32, got '{value}'"))
}

fn next_u64<I, S>(iter: &mut I, flag: &str) -> Result<u64, String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let value = next_value(iter, flag)?;
    value
        .parse::<u64>()
        .map_err(|_| format!("{flag} expected a u64, got '{value}'"))
}

#[cfg(test)]
mod tests {
    use super::{parse_args, Command};

    #[test]
    fn parses_bench_flags() {
        let cmd = parse_args([
            "bench",
            "--width",
            "128",
            "--height",
            "64",
            "--steps",
            "10",
            "--seed",
            "9",
            "--json",
            "--export-pgm",
            "out.pgm",
        ])
        .unwrap();
        match cmd {
            Command::Bench(args) => {
                assert_eq!(args.width, 128);
                assert_eq!(args.height, 64);
                assert_eq!(args.steps, 10);
                assert_eq!(args.seed, 9);
                assert!(args.json);
                assert_eq!(
                    args.export_pgm.as_ref().map(|p| p.to_str().unwrap()),
                    Some("out.pgm")
                );
            }
            _ => panic!("expected bench"),
        }
    }

    #[test]
    fn rejects_unknown_command() {
        assert!(parse_args(["nope"]).is_err());
    }
}
