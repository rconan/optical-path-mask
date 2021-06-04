const ALPHA: f64 = 13.522;

/// Optical path mask specifications
#[allow(dead_code)]
pub enum OpticalPathMaskSpecs {
    Center,
    Outer,
}

impl OpticalPathMaskSpecs {
    /// Clear aperture diameter [m]
    pub fn clear_diameter(&self) -> f64 {
        match self {
            Self::Center => 8.3576,
            Self::Outer => 8.3544,
        }
    }
    /// Clear aperture radius [m]
    pub fn clear_radius(&self) -> f64 {
        Self::clear_diameter(self) * 0.5
    }
    /// Mask origin coordinates [m]
    ///
    /// The origin of the mask is given with respect to the vertex of M1 conic surface
    pub fn vertex_origin(&self) -> [f64; 3] {
        match self {
            Self::Center => [0., 0., 0.938 - 0.572],
            Self::Outer => [
                8.848320 - 0.951 * ALPHA.to_radians().sin(),
                0.,
                0.478522 + 0.951 * ALPHA.to_radians().cos(),
            ],
        }
    }
    /// Segment tilt angle [rd]
    pub fn tilt(&self) -> f64 {
        match self {
            Self::Center => 0_f64,
            Self::Outer => ALPHA.to_radians(),
        }
    }
}
