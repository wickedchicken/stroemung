pub mod args;
pub mod cell;
pub mod grid;
pub mod math;
pub mod simulation;
pub mod types;
pub mod visualization;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use args::Args;
use grid::SimulationGrid;

pub fn run(args: Args) {
    println!("Exécute des simulations...");
    println!("Grid size {} x {}", args.x_cells, args.y_cells);
    let grid = match args.grid_file {
        Some(filename) => {
            let file = File::open(Path::new(&filename)).unwrap();
            SimulationGrid::from_reader(BufReader::new(file)).unwrap()
        }
        _ => grid::presets::empty([args.x_cells, args.y_cells]),
    };

    println!("{}", grid);
}
