use world::BiomeType;
use world::generator::layers::Layer;

pub fn generate_biome(seed: isize) -> Result<(), Box<dyn std::error::Error>> {
    let (_, mut generator) = Layer::create_generator(seed);

    let biome_count = 1;
    let width = biome_count * 16;

    let mut result = vec![0; width * width * 3];

    let mut res = generator.generate(0, 0, width, width);

    for y in 0..width {
        for x in 0..width {
            let (r, g, b) = BiomeType::from_id(*res.at(x as isize, y as isize)).color();
            let pos = (y * width + x) * 3;

            result[pos] = r;
            result[pos + 1] = g;
            result[pos + 2] = b;
        }
    }

    image::save_buffer(
        &std::path::Path::new("biomes.png"), &result, width as u32, width as u32, image::ColorType::Rgb8
    )?;

    Ok(())
}
