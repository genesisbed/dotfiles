use clap::Parser;
use serde::{Deserialize, Deserializer};
use shellexpand::tilde;
use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Debug, Display},
    fs, io,
    path::PathBuf,
    result,
};

#[derive(Debug)]
enum MeiyaError {
    Io(io::Error),
    Toml(toml::de::Error),
}

impl Display for MeiyaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Io(err) => writeln!(f, "IO error: {}", err),
            Self::Toml(err) => writeln!(f, "TOML parsing error: {}", err),
        }
    }
}

impl From<io::Error> for MeiyaError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<toml::de::Error> for MeiyaError {
    fn from(err: toml::de::Error) -> Self {
        Self::Toml(err)
    }
}

impl Error for MeiyaError {}
type Result<T> = result::Result<T, MeiyaError>;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    templates: Option<PathBuf>,
}

#[derive(Deserialize, Debug)]
struct Scheme {
    alpha: u8,
    palette: HashMap<String, String>,
}

fn expand_tilde<'de, D>(deserializer: D) -> result::Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(PathBuf::from(tilde(&s).into_owned()))
}

#[derive(Deserialize, Debug)]
struct How {
    nick: String,
    symlink: bool,
    #[serde(deserialize_with = "expand_tilde")]
    out: PathBuf,
}

fn main() -> Result<()> {
    let cfg = dirs::config_dir()
        .expect("Failed to locate a valid .config directory.")
        .join("meiya");
    if !cfg.exists() {
        fs::create_dir_all(&cfg)?;
    };

    let (s, t) = (cfg.join("scheme.toml"), cfg.join("templates"));
    s.try_exists().expect("No scheme.toml found");
    t.try_exists().expect("No templates founds.");

    let scheme: Scheme = toml::from_str(&fs::read_to_string(s)?)?;

    for entry in fs::read_dir(t)? {
        let share = entry?.path();
        let mut clean = fs::read_to_string(share.join("base"))?;
        let how: How = toml::from_str(&fs::read_to_string(share.join("how.toml"))?)?;

        for (key, value) in scheme.palette.iter() {
            let placeholder = format!("${}", key);
            clean = clean.replace(&placeholder, value);
        }
        fs::write(&how.out, clean)?;

        println!("{} (nosym) -> {:?} {}", how.nick, how.out, {
            match how.out.exists() {
                true => "[overwritten]",
                false => "[new]",
            }
        });
    }
    println!("--- Done!");

    Ok(())
}
