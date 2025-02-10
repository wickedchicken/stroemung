use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'x', default_value_t = 10)]
    x_cells: usize,

    #[arg(short = 'y', default_value_t = 5)]
    y_cells: usize,
}

fn main() {
    let args = Args::parse();

    println!("Exécute des simulations...");
    println!("Grid size {} x {}", args.x_cells, args.y_cells);
}
