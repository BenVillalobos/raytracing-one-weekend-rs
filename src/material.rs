use nalgebra::Vector3;
use rand::rngs::ThreadRng;
use crate::hittable::HitRecord;
use crate::ray::Ray;
use rand::*;

static PI: f64 = 3.1415926535897932385;
static INFINITY: f64 = f64::MAX;

fn reflect(v: Vector3<f64>, n: Vector3<f64>) -> Vector3<f64> {
    v - 2.0 * v.dot(&n)*n
}

fn deg_to_rad(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

fn clamp(x: f64, min: f64, max: f64) -> f64 {
    return if x < min { min } else if x > max { max } else { x }
}

fn random_vec3(rng: &mut ThreadRng, min: f64, max: f64) -> Vector3<f64> {
    Vector3::<f64>::new(rng.gen_range(min, max), rng.gen_range(min, max), rng.gen_range(min, max))
}

fn random_unit_vector() -> Vector3<f64> {
    random_in_unit_sphere().normalize()
}

fn random_in_hemisphere(normal: Vector3<f64>) -> Vector3<f64> {
    let in_unit_sphere = random_in_unit_sphere();
    // In the same hemisphere as the normal?
    if in_unit_sphere.dot(&normal) > 0.0 {
        in_unit_sphere
    } else {
        -in_unit_sphere
    }
}

fn random_in_unit_sphere() -> Vector3<f64> {
    let mut rng = rand::thread_rng();
    let unit = Vector3::new(1.0, 1.0, 1.0);
    loop {
        let p = 2.0 * Vector3::new(rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>()) - unit;
        if p.magnitude_squared() < 1.0 {
            return p
        }
    }
}

fn near_zero(vec: Vector3<f64>) -> bool {
    let s = 1e-8;
    return vec.x < s && vec.y < s && vec.z < s;
}

fn refract(uv: Vector3<f64>, n: Vector3<f64>, etai_over_etat: f64) -> Vector3<f64> {
    let cos_theta = clamp((-uv).dot(&n), 1.0, INFINITY);

    let r_out_perp = etai_over_etat * (uv + cos_theta*n);
    let r_out_parallel = -((1.0 - r_out_perp.magnitude_squared()).abs().sqrt()) * n;
    r_out_perp + r_out_parallel
}

pub trait Material {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<(Ray, Vector3<f64>)>;
}

pub struct Metal {
    albedo: Vector3<f64>,
    fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Vector3<f64>, fuzz: f64) -> Self {
        Metal {
            albedo: albedo,
            fuzz: if fuzz < 1.0 { fuzz } else { 1.0 },
        }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<(Ray, Vector3<f64>)> {
        let mut reflected = reflect(ray.dir.normalize(), hit.normal);
        if self.fuzz > 0.0 { reflected += self.fuzz * random_in_unit_sphere() }
        if reflected.dot(&hit.normal) > 0.0 {
            let scattered = Ray::new(hit.point, reflected);
            Some((scattered, self.albedo))
        }
        else {
            None
        }
    }
}

pub struct Lambertian {
    albedo: Vector3<f64>,
}

impl Lambertian {
    pub fn new(albedo: Vector3<f64>) -> Self {
        Lambertian {
            albedo: albedo,
        }
    }
}

impl Material for Lambertian {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<(Ray, Vector3<f64>)> {
        let mut scatter_direction = hit.normal + random_unit_vector();

        if near_zero(scatter_direction) {
            scatter_direction = hit.normal;
        }

        Some((Ray::new(hit.point, scatter_direction), self.albedo))
    }
}

pub struct Dielectric {
    ir: f64,
}

impl Dielectric {
    pub fn new(ir: f64) -> Self {
        Dielectric {
            ir: ir,
        }
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<(Ray, Vector3<f64>)> {
        let attenuation = Vector3::<f64>::new(1.0, 1.0, 1.0);

        let refraction_ratio = if hit.front_face {1.0/self.ir} else {self.ir};
        let unit_direction = ray.dir.normalize();

        let cos_theta = clamp((-unit_direction).dot(&hit.normal), 1.0, INFINITY);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let direction: Vector3<f64>;

        if cannot_refract {
            direction = reflect(unit_direction, hit.normal);
        }
        else {
            direction = refract(unit_direction, hit.normal, self.ir);
        }

        Some((Ray::new(hit.point, direction), attenuation))
    }
}