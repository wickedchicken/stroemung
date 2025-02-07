#!/usr/bin/env python3

import tempfile

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


def test_config_load():
    with tempfile.TemporaryDirectory() as temp_dir:
        config_file = Path(temp_dir, "config.json")

        with open(config_file, "w") as f:
            data = {
                "problem_type": "circle",
                "inputfile": "none",
                "xlength": 22.0,
                "ylength": 4.1,
                "imax": 110,
                "jmax": 20,
                "t_end": 60.0,
                "t_delta": 0.003,
                "vecfile": "none",
                "itermax": 100,
                "eps": 0.001,
                "omega": 1.7,
                "gamma": 0.9,
                "reynolds": 100.0,
                "ui": 1.0,
                "vi": 0.0,
            }
            json.dump(data, f)

        parsed_struct = generate_test_data.load_config_from_json(config_file)

        expected = generate_test_data.SimulationInput(
            problem_type=generate_test_data.ProblemType.CIRCLE,
            inputfile="none",
            xlength=22.0,
            ylength=4.1,
            imax=110,
            jmax=20,
            t_end=60.0,
            t_delta=0.003,
            vecfile="none",
            itermax=100,
            eps=0.001,
            omega=1.7,
            gamma=0.9,
            reynolds=100.0,
            ui=1.0,
            vi=0.0,
        )

        assert parsed_struct == expected


def test_template_generation():
    expected = Path(TEST_DATA_DIR, "template_expected.par").read_text()

    simulation_input = generate_test_data.SimulationInput(
        problem_type=generate_test_data.ProblemType.CIRCLE,
        inputfile="none",
        xlength=22.0,
        ylength=4.1,
        imax=110,
        jmax=20,
        t_end=60.0,
        t_delta=0.003,
        vecfile="none",
        itermax=100,
        eps=0.001,
        omega=1.7,
        gamma=0.9,
        reynolds=100.0,
        ui=1.0,
        vi=0.0,
    )

    with tempfile.TemporaryDirectory() as temp_dir:
        (config_filename, output_filename) = generate_test_data.generate_template(
            simulation_input, temp_dir
        )

        result = config_filename.read_text()
        result = result.replace(str(Path(temp_dir, "simulation.out")), "simulation.out")

        assert result == expected
