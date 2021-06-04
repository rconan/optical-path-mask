# GMT M1 Optical Path Mask

Computes the contour of the optical path mask for segment #1 of M1 such as the clear aperture perimeter is sampled with a resolution of 2.5cm.
The application generates a drawing of the contour in the M1-B1 coordinate system and save the contour coordinates (M1-B1) in a csv file as: y1,x1,y2,x2,...,yn,xn.

## Installation

First, install [Rust](https://www.rust-lang.org/tools/install) then install the application with 
```
cargo install --git https://github.com/rconan/optical-path-mask.git --branch main
```
