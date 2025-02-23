use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short = 'x', default_value_t = 10)]
    pub x_cells: usize,

    #[arg(short = 'y', default_value_t = 5)]
    pub y_cells: usize,

    #[arg(long)]
    pub grid_file: Option<String>,
}
