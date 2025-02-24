#!/usr/bin/env python3

import argparse
import json
import struct
from pathlib import Path

from dataclasses import dataclass
from enum import IntFlag, CONFORM
from typing import List


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
    )
    parser.add_argument(
        "--parse-outfile",
        required=True,
        type=Path,
        help="Parse an outfile from NaSt2D",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Write the JSON output to a file instead of stdout",
    )

    return parser.parse_args()


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


def main(args):
    with open(args.parse_outfile, "rb") as f:
        grid = parse_out_file(f)

    if args.output:
        with open(args.output, "w") as f:
            json.dump(grid.as_rust_grid(), f, sort_keys=2, indent=2)
    else:
        print(f"{json.dumps(grid.as_rust_grid(), sort_keys=2, indent=2)}")

    return 0


if __name__ == "__main__":
    main(get_args())
