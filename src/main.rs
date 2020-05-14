#![feature(clamp)]

#[macro_use]
extern crate clap;

mod biome_generator;
mod dump;
mod game;

use clap::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conf = load_yaml!("cli.yaml");

    let matches = App::from_yaml(conf).get_matches();

    if let Some(args) = matches.subcommand_matches("play") {
        let seed = args.value_of("seed").unwrap_or("0").parse::<isize>()?;
        let view_distance = args
            .value_of("view-distance")
            .unwrap_or("0")
            .parse::<usize>()?;

        if seed == 0 {
            //FIXME random seed ?
        }

        game::game(seed, view_distance);
    } else if let Some(args) = matches.subcommand_matches("render_chunks") {
        let seed = args.value_of("seed").unwrap_or("0").parse::<isize>()?;
        biome_generator::generate_biome(seed)?;
    } else if let Some(args) = matches.subcommand_matches("dump") {
        dump::dump_map(args)?;
    }

    Ok(())
}
