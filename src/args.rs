use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, default_value_t = 100)]
    pub x_cells: usize,

    #[arg(long, default_value_t = 20)]
    pub y_cells: usize,

    #[arg(long, default_value_t = 0.1)]
    pub x_cell_width: f64,

    #[arg(long, default_value_t = 0.2)]
    pub y_cell_height: f64,

    #[arg(long, default_value_t = 0.005)]
    pub delta_t: f64,

    #[arg(long, default_value_t = 0.9)]
    pub gamma: f64,

    #[arg(long, default_value_t = 100.0)]
    pub reynolds: f64,

    #[arg(long, default_value_t = 0.001)]
    pub sor_epsilon: f64,

    #[arg(long, default_value_t = 100)]
    pub sor_max_iterations: u32,

    #[arg(long, default_value_t = 1.7)]
    pub omega: f64,

    #[arg(long)]
    pub sim_file: Option<String>,
}
