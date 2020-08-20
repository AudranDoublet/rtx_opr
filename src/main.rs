#![feature(clamp)]

#[macro_use]
extern crate clap;

mod biome_generator;
mod dump;
mod game;
mod termidraw;

use clap::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conf = load_yaml!("cli.yaml");

    let matches = App::from_yaml(conf).get_matches();

    if let Some(args) = matches.subcommand_matches("play") {
        let seed = args.value_of("seed").unwrap_or("0").parse::<isize>()?;
        let view_distance = args
            .value_of("view-distance")
            .unwrap_or("10")
            .parse::<usize>()?;
        let resolution_coeff = args
            .value_of("resolution-coeff")
            .unwrap_or("1")
            .parse::<f32>()?
            .max(0.125)
            .min(10.0);
        let world_path = args.value_of("world").unwrap_or("worldp");
        let layout = game::Layout::parse(args.value_of("layout").unwrap_or("fr"));
        let with_shadows = true; //args.is_present("with-shadows");
        let flat = args.is_present("flat");

        if seed == 0 {
            //FIXME random seed ?
        }

        game::game(
            world_path,
            seed,
            flat,
            view_distance,
            with_shadows,
            resolution_coeff,
            layout,
        )?;
    } else if let Some(args) = matches.subcommand_matches("render_chunks") {
        let seed = args.value_of("seed").unwrap_or("0").parse::<isize>()?;
        biome_generator::generate_biome(seed)?;
    } else if let Some(args) = matches.subcommand_matches("dump") {
        dump::dump_map(args)?;
    }

    Ok(())
}
