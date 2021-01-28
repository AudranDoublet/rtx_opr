use nalgebra::Vector3;

/**
 * re-implemnation in rust of shaders/initial/miss.rmiss
 * this is used to compute the exact sunlight color for light scattering
 */

const CST_SKY_NUM_SAMPLES: usize = 16;
const CST_SKY_NUM_SAMPLES_LIGHT: usize = 8;
const CST_SKY_EARTH_RADIUS: f32 = 6360000.0;
const CST_SKY_ATMOSPHERE_RADIUS: f32 = 6420000.0;
const CST_SKY_ATMOSPHERE_RADIUS2: f32 = CST_SKY_ATMOSPHERE_RADIUS*CST_SKY_ATMOSPHERE_RADIUS;
const CST_SKY_HR: f32 = 7994.0;
const CST_SKY_HM: f32 = 1200.0;
const CST_SUN_INTENSITY: f32 = 20.0;

fn ray_atmosphere_intersect(orig: Vector3<f32>, dir: Vector3<f32>) -> Option<(f32, f32)> {
    let tca = -orig.dot(&dir);
    let d2 = orig.dot(&orig) - tca*tca;

    if d2 > CST_SKY_ATMOSPHERE_RADIUS2 {
        None
    } else {
        let thc = (CST_SKY_ATMOSPHERE_RADIUS2 - d2).sqrt();

        Some(((tca - thc).max(0.0), tca + thc))
    }
}

pub fn compute_sky_light(mut dir: Vector3<f32>, sun_direction: Vector3<f32>) -> Vector3<f32>
{
    let cst_sky_beta_r: Vector3<f32> = Vector3::new(3.8e-6, 13.5e-6, 33.1e-6); 
    let cst_sky_beta_m: Vector3<f32> = Vector3::new(21e-6, 21e-6, 21e-6);

    if dir.y < 0.1 {
        dir.y = 0.1;
    }

    let orig = Vector3::new(0.0, CST_SKY_EARTH_RADIUS, 0.0);

    let (tmin, tmax) = match ray_atmosphere_intersect(orig, dir) {
        Some(v) => v,
        None => return Vector3::zeros(),
    };

    // mie and rayleigh contribution
    let mut sum_r = Vector3::zeros();
    let mut sum_m = Vector3::zeros();

    let segment_length = (tmax - tmin) / CST_SKY_NUM_SAMPLES as f32;
    let mut t_current = tmin;

    let mut optical_depth_r = 0.0;
    let mut optical_depth_m = 0.0;
    let mu = dir.dot(&(-sun_direction)); // mu in the paper which is the cosine of the angle between the sun direction and the ray direction
    let phase_r = 3.0 / (16.0 * std::f32::consts::PI) * (1.0 + mu * mu);
    let g = 0.76;
    let phase_m = 3.0 / (8.0 * std::f32::consts::PI) * ((1.0 - g * g) * (1.0 + mu * mu)) / ((2.0 + g * g) * (1.0 + g * g - 2.0 * g * mu).powf(1.5));

    for _ in 0..CST_SKY_NUM_SAMPLES {
        let sample_position = orig + (t_current + segment_length * 0.5) * dir;
        let height = sample_position.norm() - CST_SKY_EARTH_RADIUS;

        // compute optical depth for light
        let hr = (-height / CST_SKY_HR).exp() * segment_length;
        let hm = (-height / CST_SKY_HM).exp() * segment_length;
        optical_depth_r += hr;
        optical_depth_m += hm;

        // light optical depth
        let (_, t1_light) = ray_atmosphere_intersect(sample_position, -sun_direction).unwrap_or((0.0, 0.0));

        let segment_length_light = t1_light / CST_SKY_NUM_SAMPLES_LIGHT as f32;
        let mut t_current_light = 0.0;
        let mut optical_depth_light_r = 0.0;
        let mut optical_depth_light_m = 0.0;

        let mut end = false;
        for j in 0..CST_SKY_NUM_SAMPLES_LIGHT {
            let sample_position_light = sample_position + (t_current_light + segment_length_light * 0.5) * (-sun_direction);
            let height_light = sample_position_light.norm() - CST_SKY_EARTH_RADIUS;

            if height_light < 0.0 {
                break;
            }

            end = j == CST_SKY_NUM_SAMPLES_LIGHT - 1;

            optical_depth_light_r += (-height_light / CST_SKY_HR).exp() * segment_length_light;
            optical_depth_light_m += (-height_light / CST_SKY_HM).exp() * segment_length_light;
            t_current_light += segment_length_light;
        }

        if end {
            let tau = cst_sky_beta_r * (optical_depth_r + optical_depth_light_r) + cst_sky_beta_m * 1.1 * (optical_depth_m + optical_depth_light_m);
            let attenuation = Vector3::new((-tau.x).exp(), (-tau.y).exp(), (-tau.z).exp());
            sum_r += attenuation * hr;
            sum_m += attenuation * hm;
        }

        t_current += segment_length;
    }

    return (sum_r.component_mul(&cst_sky_beta_r) * phase_r + sum_m.component_mul(&cst_sky_beta_m) * phase_m) * CST_SUN_INTENSITY;
}
