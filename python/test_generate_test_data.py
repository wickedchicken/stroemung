#!/usr/bin/env python3

from pathlib import Path


import generate_test_data

import json


TEST_DATA_DIR = Path(Path(__file__).parent.resolve(), "test_data")


def load_file(filename):
    with open(filename, "r") as f:
        return json.load(f)


def load_expected_simulation(filename):
    data = load_file(filename)

    # Postprocess the flags field
    data["flags"] = [[generate_test_data.Flag(v) for v in x] for x in data["flags"]]

    return generate_test_data.SimulationOutput(**data)


def test_data_parsing():
    filename = "small_data.out"

    with open(Path(TEST_DATA_DIR, filename), "rb") as f:
        parsed_output = generate_test_data.parse_out_file(f)

    expected = load_expected_simulation(
        Path(TEST_DATA_DIR, filename + "_expected.json")
    )

    assert parsed_output == expected


def test_rust_grid_output():
    filename = "small_data.out"

    with open(Path(TEST_DATA_DIR, filename), "rb") as f:
        parsed_output = generate_test_data.parse_out_file(f).as_rust_grid()

    expected = load_file(Path(TEST_DATA_DIR, filename + "_rust_expected.json"))

    assert parsed_output == expected
