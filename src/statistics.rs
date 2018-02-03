use cs;

use std::fmt;

pub struct Statistics {
    samples_count: u32,
    average_gpu_stats: cs::ty::Statistics,
    average_render_time: i64,
    triangle_count: usize,
    primary_rays: u32,
}

impl Statistics {
    pub fn new(triangle_count: usize, primary_rays: u32) -> Statistics {
        let average_gpu_stats = cs::ty::Statistics {
            triangle_tests: 0,
            triangle_intersections: 0,
        };
        Statistics {
            samples_count: 0,
            average_gpu_stats,
            average_render_time: 0,
            triangle_count,
            primary_rays,
        }
    }

    pub fn add_stats(&mut self, gpu_stats: &cs::ty::Statistics, render_time: i64) {
        self.average_gpu_stats.triangle_tests = (self.average_gpu_stats.triangle_tests
            * self.samples_count
            + gpu_stats.triangle_tests)
            / (self.samples_count + 1);
        self.average_gpu_stats.triangle_intersections =
            (self.average_gpu_stats.triangle_intersections * self.samples_count
                + gpu_stats.triangle_intersections) / (self.samples_count + 1);
        self.average_render_time = (self.average_render_time * self.samples_count as i64
            + render_time) / (self.samples_count as i64 + 1);
        self.samples_count += 1;
    }
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Render time: {} ms\n", self.average_render_time)?;
        write!(f, "Triangles: {}\n", self.triangle_count)?;
        write!(f, "Primary rays: {}\n", self.primary_rays)?;
        write!(
            f,
            "Triangle tests: {}\n",
            self.average_gpu_stats.triangle_tests
        )?;
        write!(
            f,
            "Triangle intersections: {}\n",
            self.average_gpu_stats.triangle_intersections
        )?;
        write!(
            f,
            "Tests per ray: {}\n",
            self.average_gpu_stats.triangle_tests as f32 / self.primary_rays as f32
        )?;
        write!(
            f,
            "Tests per triangle: {}",
            self.average_gpu_stats.triangle_tests as f32 / self.triangle_count as f32
        )?;
        Ok(())
    }
}
