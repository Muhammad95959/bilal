use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(
    name = "bilal",
    version,
    about = "Bilal [A CLI salah time]",
    after_long_help = "Bugs can be reported on GitHub: https://github.com/azzamsa/bilal/issues"
)]
pub struct Opts {
    /// A Salah mode to show
    #[arg(value_enum)]
    pub mode: Mode,

    /// Display Salah in JSON formatted string
    #[arg(short = 'J', long)]
    pub json: bool,

    /// Display Salah in colored output
    #[arg(
        short,
        long,
        value_enum,
        default_value_t = Color::Auto,
    )]
    pub color: Color,
}

#[derive(Clone, ValueEnum)]
pub enum Mode {
    All,
    Next,
    Current,
}

#[derive(Clone, ValueEnum)]
pub enum Color {
    /// show colors if the output goes to an interactive console (default)
    Auto,
    /// always use colorized output
    Always,
    /// do not use colorized output
    Never,
}

impl Color {
    pub fn as_str(&self) -> &'static str {
        match self {
            Color::Auto => "auto",
            Color::Never => "never",
            Color::Always => "always",
        }
    }
}
