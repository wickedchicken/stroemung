use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use stroemung::grid::SimulationGrid;

fn load_test_file(filename: &str) -> BufReader<File> {
    let test_data_directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_data");
    let test_filename = test_data_directory.join(filename);
    BufReader::new(File::open(test_filename).unwrap())
}

#[test]
fn deserialize() {
    let result: SimulationGrid =
        serde_json::from_reader(load_test_file("small_data.out.json")).unwrap();
    insta::assert_json_snapshot!(result);
}
