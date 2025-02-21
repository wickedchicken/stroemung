pub mod args;
pub mod cell;
pub mod grid;
pub mod math;
pub mod types;

use args::Args;

pub fn run(args: Args) {
    println!("Exécute des simulations...");
    println!("Grid size {} x {}", args.x_cells, args.y_cells);
    let grid = grid::presets::zeroes([args.x_cells, args.y_cells]);
    println!("{}", grid);
}
