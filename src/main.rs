mod lib;
use csv;
use gmt_analytics::{new_ray, Arithmetic, Conic, Plane, Ray, RayTracing};
use lib::OpticalPathMaskSpecs;
use plotters::prelude::*;
use std::{error::Error, fs::File};

pub fn local_radius(ray: &Ray, mask_specs: &OpticalPathMaskSpecs) -> (f64, f64) {
    let u = ray.p.sub(mask_specs.vertex_origin());
    let (s, c) = mask_specs.tilt().sin_cos();
    let v = [c * u[0] + s * u[2], u[1], -s * u[0] + c * u[2]];
    //println!("{:+.6?}", v);
    (v[0], v[1])
}

pub fn print_rays(rays: Option<&[Ray]>, title: Option<&str>) {
    if let Some(rays) = rays {
        if let Some(title) = title {
            println!("{}", title);
        }
        rays.iter().for_each(|ray| println!("{}", ray));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Select the segment
    let mask_specs = OpticalPathMaskSpecs::Outer;
    let clear_aperture_radius = mask_specs.clear_radius();
    println!("Clear aperture radius: {}m", clear_aperture_radius);
    println!("Mask origin: {:#?}m", mask_specs.vertex_origin());

    // Starting height of the incoming rays (on-axis source)
    let z0 = 3f64;
    // Clear aperture perimeter sampling
    let step = Some(2.5e-2);
    let n_ray = step.map_or_else(
        || 4i32,
        |x: f64| (2f64 * std::f64::consts::PI * clear_aperture_radius / x).ceil() as i32,
    );
    println!("Rays #: {}", n_ray);

    // Ray bundle
    let mut rays: Vec<_> = (0..n_ray + 1)
        .map(|i| {
            // sine & cosine of the polar angle
            let (s, c) = (2. * std::f64::consts::PI * (i as f64 / n_ray as f64)).sin_cos();
            let radius = clear_aperture_radius;
            // clear aperture perimeter cartesian coordinates
            let (x, y) = (c * radius, s * radius);
            match mask_specs {
                OpticalPathMaskSpecs::Center => new_ray().point_of_origin([x, y, z0]).build(),
                OpticalPathMaskSpecs::Outer => {
                    let origin = mask_specs.vertex_origin();
                    let (s, c) = mask_specs.tilt().sin_cos();
                    // Rays orthogonal to tilted mask
                    let mut ray_builder = new_ray().point_of_origin([
                        c * x + origin[0],
                        y + origin[1],
                        s * x + origin[2],
                    ]);
                    ray_builder.u = [s, 0., -c];
                    let mut ray = ray_builder.build();
                    // Compute intersection between the mirror and the mask normals
                    let m1 = Conic::gmt_m1();
                    ray.trace(m1.distance(&ray));
                    //println!("{}", ray);
                    // Dets the ray direction parallel to optical axis
                    ray.u = [0., 0., -1.];
                    ray.p[2] = z0;
                    ray
                }
            }
            /*
            let origin = mask_specs.vertex_origin();
            */
        })
        .collect();
    print_rays(None, Some("Rays"));

    // Sets the plane where the mask are (point, normal vector)
    let mask = Plane::new(
        mask_specs.vertex_origin(),
        [-mask_specs.tilt().sin(), 0., mask_specs.tilt().cos()],
    );
    // ray trace to mask
    mask.traces(&mut rays);
    print_rays(None, Some("Mask"));
    //println!("Mask (local)");
    //rays.iter()
    //    .for_each(|ray| println!("{:+.6?}", ray.p.sub(mask_specs.vertex_origin())));
    // intersection coordinates in mask coordinate system
    let in_mask_intersect: Vec<_> = rays
        .iter()
        .map(|ray| local_radius(ray, &mask_specs))
        .collect();

    // M1 conic
    let m1 = Conic::gmt_m1();
    //let m1_vertex = m1.height_at([8.71, 0., 0.]);
    //println!("M1 height: {:?}", m1_vertex);
    //println!("{:?}", m1.distances(&rays));
    // ray trace to conic
    m1.traces(&mut rays);
    print_rays(None, Some("M1"));

    /*
    let m1_radius: Vec<_> = rays
        .iter()
        .map(|ray| local_radius(ray, &mask_specs))
        .collect();
    println!("M1 radius [{:.5}]: {:.5?}", 8.365 / 2., m1_radius);
     */
    // reflection from M1
    rays.iter_mut().for_each(|ray| {
        m1.reflect(ray);
    });
    print_rays(None, Some("Reflect"));
    // ray trace to mask
    mask.traces(&mut rays);
    print_rays(None, Some("Mask"));
    // intersection coordinates in mask coordinate system
    let out_mask_intersect: Vec<_> = rays
        .iter()
        .map(|ray| local_radius(ray, &mask_specs))
        .collect();

    // union of the 2 mask intersects
    let mask_perimeter: Vec<_> = in_mask_intersect
        .iter()
        .zip(out_mask_intersect.iter())
        .map(|(&(xi, yi), &(xo, yo))| {
            let ri = xi.hypot(yi);
            let ro = xo.hypot(yo);
            if ri >= ro {
                (xi, yi)
            } else {
                (xo, yo)
            }
        })
        .collect();
    /*
        /*for (&(xi, yi), &(xo, yo)) in in_mask_intersect.iter().zip(out_mask_intersect.iter()) {
            let ri = xi.hypot(yi);
            let ro = xo.hypot(yo);
            println!("{:.4}/{:.4}", ri, ro);
        }*/
    mask_perimeter
        .iter()
    .for_each(|&(x, y)| println!("{}", x.hypot(y)));*/

    println!("Drawing mask contour in mask_contour.svg");
    let root_area = SVGBackend::new("mask_contour.svg", (600, 600)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let l = 4.5;
    let mut ctx = ChartBuilder::on(&root_area)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .margin(10)
        .build_cartesian_2d(-l..l, -l..l)?;

    ctx.configure_mesh()
        .x_desc("M1-B1 y [m]")
        .y_desc("M1-B1 x [m]")
        .draw()?;

    ctx.draw_series(LineSeries::new(mask_perimeter.iter().copied(), &BLACK))?;

    let file = File::create("mask_contour.csv")?;
    let mut wtr = csv::Writer::from_writer(file);
    wtr.serialize(mask_perimeter)?;
    wtr.flush()?;
    Ok(())
}
