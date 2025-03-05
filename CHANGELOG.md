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

## [0.1.0] - 2025-02-04

### Added

- Initial empty release based off of the https://github.com/rust-github/template
  template.
