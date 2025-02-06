#!/usr/bin/env python3

import argparse

def get_args():
    parser = argparse.ArgumentParser(
        description='Run NaSt2D and generate test files',
    )

    return parser.parse_args()

def main(args):
    return 0

if __name__ == '__main__':
    main(get_args())