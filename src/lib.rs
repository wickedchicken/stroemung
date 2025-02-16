pub mod args;
pub mod cell;
pub mod grid;
pub mod math;
pub mod simulation;
pub mod types;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use args::Args;

pub fn run(args: Args) {
    println!("Exécute des simulations...");
    println!("Grid size {} x {}", args.x_cells, args.y_cells);
    let grid = match args.grid_file {
        Some(filename) => {
            let file = File::open(Path::new(&filename)).unwrap();
            serde_json::from_reader(BufReader::new(file)).unwrap()
        }
        _ => grid::presets::empty([args.x_cells, args.y_cells]),
    };

    println!("{}", grid);
}
