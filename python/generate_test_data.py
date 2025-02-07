#!/usr/bin/env python3

import argparse
import struct

from dataclasses import dataclass
from typing import List

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
    flags: List[List[int]]  # Flags


def get_args():
    parser = argparse.ArgumentParser(
        description="Run NaSt2D and generate test files",
    )

    return parser.parse_args()


def parse_array(stream, imax, jmax):
    # For each row, take a column of numbers from the stream, put them into a
    # list, and build the outer row list out of those lists.
    return [
      [next(stream) for _ in range(jmax + 2)]
      for _ in range(imax + 2)
    ]


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
    flags = parse_array(int_stream, imax, jmax)

    return SimulationOutput(
        imax,
        jmax,
        U,
        V,
        P,
        T,
        flags,
    )


def parse_out_file(file_obj, int_bytes=4, int_type='i'):
    """
    Parse the raw bytes from a NaSt2D .out file into a SimulationOutput object.
    """
    # Create a generator that yields floats from the raw file_obj bytestream.
    def float_stream():
        # NaSt2D outfiles use C doubles, which are 8 bytes most of the time.
        while len(byte_chunk := file_obj.read(8)) == 8:
            # Parse the 8 bytes as a float
            yield struct.unpack('d', byte_chunk)[0]

    # Create a generator that yields ints from the raw file_obj bytestream.
    def int_stream():
        # NaSt2D outfiles use C ints, which are 4 bytes on my machine.
        # The size and type can be overridden in the function signature
        # if sizeof(int) differs on the machine running NaSt2D.

        while len(byte_chunk := file_obj.read(int_bytes)) == int_bytes:
            # Parse the bytes as an int
            yield struct.unpack(int_type, byte_chunk)[0] 

    return parse_stream(int_stream(), float_stream())


def main(args):
    return 0


if __name__ == "__main__":
    main(get_args())
