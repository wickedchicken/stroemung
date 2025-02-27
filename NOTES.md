# Notes

## Overall

* Could declare and use array types a bit more.

## math.rs

* Written using ArrayView instead of ArrayBase, the templating got weird.
  Someone could make this work though.
* du2dx and dv2dy are identical, we could write them as the same function that slices
  the 3x3 ArrayView into what's needed.
* Possibly could do the same for duvdx and duvdx if you thought hard enough about it.
* Tests in math.rs use some duplicate data, would be nice to make a common function for that.
* Currently linear algebra routines are written all in one big iteration. We might be
  able to use smaller piecewise functions though (for example, doing the division by
  4.0*delx or 4.0*dely as a separate operation at the end, BLAS-style). Not sure if
  that would buy any performance or just make things more complicated for no reason.
* Laplace operators are currently implemented with array accesses as per the book.
  We might be able to use a convolution kernel instead to make things
  simpler.
* In theory you could precompute 4.0 * delx and pass that through all the algebra
  functions, but it would be annoying. Would be interesting to see if that sped
  anything up.
* We're comparing binary floating point numbers using decimal representation.
  There could be small bits of weirdness.
