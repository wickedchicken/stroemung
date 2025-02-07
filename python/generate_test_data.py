#!/usr/bin/env python3

import argparse
import json
import struct
import subprocess
import tempfile
from pathlib import Path

from dataclasses import asdict, dataclass
from enum import auto, IntFlag, CONFORM, StrEnum
from typing import List

from jinja2 import Environment, PackageLoader, select_autoescape

JINJA_ENV = Environment(
    loader=PackageLoader("generate_test_data", "templates"),
    keep_trailing_newline=True,
    autoescape=select_autoescape(),
)


class ProblemType(StrEnum):
    CIRCLE = auto()


class Flag(IntFlag, boundary=CONFORM):
    BOUNDARY = 0
    FLUID = 16

    def as_rust_cell(self, inflow=None, outflow=False):
        if self is Flag.FLUID:
            return "Fluid"

        if inflow is not None:
            return {"Boundary": {"Inflow": {"velocity": inflow}}}

        if outflow:
            return {"Boundary": "Outflow"}

        return {"Boundary": "NoSlip"}


@dataclass
class SimulationOutput:
    imax: int  # Horizontal array size of inner simulation (excluding boundary)
    jmax: int  # Vertical array size of inner simulation (excluding boundary)

    # The following matrices are row-major indexed, meaning
    # they are accessed as U[x][y].
    U: List[List[float]]  # Velocity in the x-dimension
    V: List[List[float]]  # Velocity in the y-dimension
    P: List[List[float]]  # Pressure
    T: List[List[float]]  # Temperature
    flags: List[List[Flag]]  # Flags

    def as_rust_grid(self):
        size = [self.imax + 2, self.jmax + 2]

        def flatten_array(arr):
            return {"v": 1, "dim": size, "data": [x for y in arr for x in y]}

        # NaSt2D doesn't output what type of boundary cell a cell is. We have
        # to use knowledge of how its presets are generated to reconstruct this
        # data. In the preset files we use, the left wall is composed of
        # Inflow cells, the right wall Outflow cells, and the top and bottom
        # walls NoSlip cells. All interior boundaries (meaning: inside the
        # simulation grid as opposed to the walls) are NoSlip cells.
        new_flags = []
        for x, (ux, vx, flagsx) in enumerate(zip(self.U, self.V, self.flags)):
            row = []
            for y, (uval, vval, flagval) in enumerate(zip(ux, vx, flagsx)):
                inflow = None
                outflow = False
                if y > 0 and y <= self.jmax:
                    if x == 0:
                        inflow = [uval, vval]
                    elif x == self.imax + 1:
                        outflow = True
                row.append(flagval.as_rust_cell(inflow=inflow, outflow=outflow))
            new_flags.append(row)

        return {
            "size": size,
            "u": flatten_array(self.U),
            "v": flatten_array(self.V),
            "pressure": flatten_array(self.P),
            "cell_type": flatten_array(new_flags),
        }


def get_args():
    parser = argparse.ArgumentParser(
        description="Run NaSt2D and generate test files",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )

    execution_mode_group = parser.add_argument_group(
        "Execution Mode",
        "Determine what generate_test_data.py does and how it loads its data.",
    )
    group = execution_mode_group.add_mutually_exclusive_group(required=True)
    group.add_argument(
        "--parse-outfile",
        type=Path,
        help="Parse an outfile from NaSt2D instead of running the program",
    )

    group.add_argument(
        "--config-file",
        type=Path,
        help="Load simulation parameters from a JSON configuration file.",
    )

    group.add_argument(
        "--cli-config",
        action="store_true",
        help="Load simulation parameters from CLI flags.",
    )

    parser.add_argument(
        "--nast2d",
        type=Path,
        help="Path to the NaSt2D executable to run",
    )

    parser.add_argument(
        "--output",
        type=Path,
        help="Write the JSON output to a file instead of stdout",
    )

    simulation_group = parser.add_argument_group(
        "Simulation Options",
        "Options for the simulation if running NaSt2D",
    )

    simulation_group.add_argument(
        "--sim-problem-type",
        choices=[x.value.lower() for x in ProblemType],
        default=ProblemType.CIRCLE,
        type=ProblemType,
        help="Which problem to run",
    )

    simulation_group.add_argument(
        "--sim-inputfile",
        type=str,
        default="none",
        help="Path to read starting values from or 'none'",
    )
    simulation_group.add_argument(
        "--sim-xlength",
        type=float,
        default=22.0,
        help="Width of simulation space (excluding boundary cells)",
    )
    simulation_group.add_argument(
        "--sim-ylength",
        type=float,
        default=22.0,
        help="Height of simulation space (excluding boundary cells)",
    )
    simulation_group.add_argument(
        "--sim-imax",
        type=int,
        default=110,
        help="Number of cells in the x-dimension (excluding boundary cells)",
    )
    simulation_group.add_argument(
        "--sim-jmax",
        type=int,
        default=110,
        help="Number of cells in the y-dimension (excluding boundary cells)",
    )
    simulation_group.add_argument(
        "--sim-t-end",
        type=float,
        default=60.0,
        help="How many simulated seconds to run simulation before terminating",
    )
    simulation_group.add_argument(
        "--sim-t-delta",
        type=float,
        default=0.003,
        help="Seconds per step (could be reduced if adaptive stepsize is activated)",
    )
    simulation_group.add_argument(
        "--sim-vecfile",
        type=str,
        default="none",
        help="Where to write visualization values to or 'none'",
    )
    simulation_group.add_argument(
        "--sim-itermax",
        type=int,
        default=100,
        help="Maximum SOR iterations per cycle",
    )
    simulation_group.add_argument(
        "--sim-eps",
        type=float,
        default=0.001,
        help="Absolute epsilon between SOR iterations to achieve convergence",
    )
    simulation_group.add_argument(
        "--sim-omega",
        type=float,
        default=1.7,
        help="SOR overrelaxation factor",
    )
    simulation_group.add_argument(
        "--sim-gamma",
        type=float,
        default=1.7,
        help="The diffusion coefficient in the upwind differencing scheme",
    )
    simulation_group.add_argument(
        "--sim-reynolds",
        type=float,
        default=100.0,
        help="The Reynolds number of the fluid",
    )
    simulation_group.add_argument(
        "--sim-ui",
        type=float,
        default=1.0,
        help="Initial u-velocity of all fluid cells",
    )
    simulation_group.add_argument(
        "--sim-vi",
        type=float,
        default=0.0,
        help="Initial v-velocity of all fluid cells",
    )

    args = parser.parse_args()

    if args.parse_outfile and args.nast2d:
        parser.error("Can't use --parse-outfile and --nast2d together")

    if args.config_file and not args.nast2d:
        parser.error("--config-file requires --nast2d as well")

    if args.cli_config and not args.nast2d:
        parser.error("--cli-config requires --nast2d as well")

    return args


def parse_array(stream, imax, jmax):
    # For each row, take a column of numbers from the stream, put them into a
    # list, and build the outer row list out of those lists.
    return [[next(stream) for _ in range(jmax + 2)] for _ in range(imax + 2)]


def parse_stream(int_stream, float_stream):
    """
    Parse the stream of numbers from a NaSt2D .out file.
    """
    imax = next(int_stream)
    jmax = next(int_stream)
    U = parse_array(float_stream, imax, jmax)
    V = parse_array(float_stream, imax, jmax)
    P = parse_array(float_stream, imax, jmax)
    T = parse_array(float_stream, imax, jmax)

    # Create a generator that yields flags from the int bytestream.
    def flag_stream():
        for value in int_stream:
            yield Flag(value)

    flags = parse_array(flag_stream(), imax, jmax)

    return SimulationOutput(
        imax,
        jmax,
        U,
        V,
        P,
        T,
        flags,
    )


def parse_out_file(file_obj, int_bytes=4, int_type="i"):
    """
    Parse the raw bytes from a NaSt2D .out file into a SimulationOutput object.

    NaSt2D outfiles use C ints, which are 4 bytes on my machine.
    The size and type can be overridden via int_bytes and int_type
    if sizeof(int) differs on the machine running NaSt2D.
    """

    # Create a generator that yields floats from the raw file_obj bytestream.
    def float_stream():
        # NaSt2D outfiles use C doubles, which should be 8 bytes.
        while len(byte_chunk := file_obj.read(8)) == 8:
            # Parse the 8 bytes as a float
            yield struct.unpack("d", byte_chunk)[0]

    # Create a generator that yields ints from the raw file_obj bytestream.
    def int_stream():
        while len(byte_chunk := file_obj.read(int_bytes)) == int_bytes:
            # Parse the bytes as an int
            yield struct.unpack(int_type, byte_chunk)[0]

    return parse_stream(int_stream(), float_stream())


@dataclass
class SimulationInput:
    problem_type: ProblemType
    inputfile: str
    xlength: float
    ylength: float
    imax: int
    jmax: int
    t_end: float
    t_delta: float
    vecfile: str
    itermax: int
    eps: float
    omega: float
    gamma: float
    reynolds: float
    ui: float
    vi: float


def generate_template(simulation_input, temp_dir):
    template = JINJA_ENV.get_template("config.par.j2")
    config_filename = Path(temp_dir, "configuration.par")
    output_filename = Path(temp_dir, "simulation.out")
    with open(config_filename, "w") as config_file:
        template.stream(
            outputfile=output_filename,
            **asdict(simulation_input),
        ).dump(config_file)
    return (config_filename, output_filename)


def call_nast2d(simulation_input, nast2d):
    with tempfile.TemporaryDirectory() as temp_dir:
        (config_filename, output_filename) = generate_template(
            simulation_input, temp_dir
        )
        print(config_filename.read_text())
        subprocess.run(
            [nast2d, config_filename],
            check=True,
        )
        with open(output_filename, "rb") as f:
            return parse_out_file(f)


def load_config_from_json(filename):
    with open(filename, "rb") as f:
        data = json.load(f)

    return SimulationInput(
        problem_type=data["problem_type"],
        inputfile=data["inputfile"],
        xlength=data["xlength"],
        ylength=data["ylength"],
        imax=data["imax"],
        jmax=data["jmax"],
        t_end=data["t_end"],
        t_delta=data["t_delta"],
        vecfile=data["vecfile"],
        itermax=data["itermax"],
        eps=data["eps"],
        omega=data["omega"],
        gamma=data["gamma"],
        reynolds=data["reynolds"],
        ui=data["ui"],
        vi=data["vi"],
    )


def main(args):
    if args.parse_outfile:
        with open(args.parse_outfile, "rb") as f:
            grid = parse_out_file(f)
    else:
        if args.config_file:
            simulation_input = load_config_from_json(args.parse_configfile)
        else:
            simulation_input = SimulationInput(
                problem_type=args.sim_problem_type,
                inputfile=args.sim_inputfile,
                xlength=args.sim_xlength,
                ylength=args.sim_ylength,
                imax=args.sim_imax,
                jmax=args.sim_jmax,
                t_end=args.sim_t_end,
                t_delta=args.sim_t_delta,
                vecfile=args.sim_vecfile,
                itermax=args.sim_itermax,
                eps=args.sim_eps,
                omega=args.sim_omega,
                gamma=args.sim_gamma,
                reynolds=args.sim_reynolds,
                ui=args.sim_ui,
                vi=args.sim_vi,
            )
        grid = call_nast2d(simulation_input, args.nast2d)

    if args.output:
        with open(args.output, "w") as f:
            json.dump(grid.as_rust_grid(), f, sort_keys=True, indent=2)
    else:
        print(f"{json.dumps(grid.as_rust_grid(), sort_keys=True, indent=2)}")

    return 0


if __name__ == "__main__":
    main(get_args())
