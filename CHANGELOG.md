# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Ability to load simulation grids from a serde file.
- Ability to parse simulation grids from NaSt2D output files and output in
  serde-parsable format.
- Basic linear algebra functions (du2dx, duvdx, duvdy, dv2dy, Laplacian) needed
  for higher-level calculations.
- Functions to calculate F and G, the horizontal and vertical non-pressure parts of the
  discretized momentum equation.
- Setup of U and V boundary conditions
- Computation of the right-hand-side of the Poisson pressure equation
- Computation of the L2 norm of the pressure residual
- Ability to solve the Poisson pressure equation via successive over-relaxation (SOR).
- Computation of new U and V values based on pressure solution
- "Inflow" grid preset, which is an empty fluid grid with upper and lower no-slip
  boundaries, an inflow boundary on the left and an outflow boundary on the right.
- Function to advance the simulation state by one iteration ("tick").
- "Obstacle" grid preset, which is a fluid grid with upper and lower no-slip
  boundaries, an inflow boundary on the left, an outflow boundary on the right, and
  a circle no-slip obstacle boundary near the left side of the simulation grid.

## [0.1.0] - 2025-02-04

### Added

- Initial empty release based off of the https://github.com/rust-github/template
  template.
