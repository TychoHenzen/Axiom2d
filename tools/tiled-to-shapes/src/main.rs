use std::path::PathBuf;

use tiled_to_shapes::{
    codegen::generate_tileset_code,
    pipeline::{convert_tileset, default_convert_config},
    scaffold::scaffold_tsx,
    tsx_parser::parse_tsx,
};

fn print_usage() {
    eprintln!(
        "Usage: tiled-to-shapes <input.tsx> [OPTIONS]\n\
         \n\
         Arguments:\n\
           <input.tsx>       Path to Tiled TSX tileset file\n\
         \n\
         Options:\n\
           -o, --output      Output directory for generated Rust file(s) [default: stdout]\n\
           --scaffold        Write missing default properties back into the TSX file\n\
           --fn-name NAME    Name for the generated tileset function [default: tileset]\n\
           --dry-run         Parse and validate without generating code\n\
           --help            Show this help message"
    );
}

struct Args {
    input: PathBuf,
    output: Option<PathBuf>,
    scaffold: bool,
    fn_name: String,
    dry_run: bool,
}

fn parse_args(args: Vec<String>) -> Result<Args, String> {
    let mut iter = args.into_iter().skip(1); // skip program name
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut scaffold = false;
    let mut fn_name = "tileset".to_owned();
    let mut dry_run = false;

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            "-o" | "--output" => {
                output = Some(PathBuf::from(
                    iter.next().ok_or("--output requires a path argument")?,
                ));
            }
            "--scaffold" => scaffold = true,
            "--dry-run" => dry_run = true,
            "--fn-name" => {
                fn_name = iter.next().ok_or("--fn-name requires a name argument")?;
            }
            other if !other.starts_with('-') => {
                if input.is_none() {
                    input = Some(PathBuf::from(other));
                } else {
                    return Err(format!("unexpected argument: {other}"));
                }
            }
            other => return Err(format!("unknown option: {other}")),
        }
    }

    let input = input.ok_or("missing required argument: <input.tsx>")?;
    Ok(Args {
        input,
        output,
        scaffold,
        fn_name,
        dry_run,
    })
}

fn main() {
    let args_vec: Vec<String> = std::env::args().collect();

    if args_vec.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let args = match parse_args(args_vec) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            print_usage();
            std::process::exit(1);
        }
    };

    // Scaffold first if requested
    if args.scaffold {
        let tileset = match parse_tsx(&args.input) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error parsing TSX: {e}");
                std::process::exit(1);
            }
        };
        match scaffold_tsx(&args.input, &tileset) {
            Ok(true) => eprintln!(
                "[tiled-to-shapes] Scaffolded missing properties into {}",
                args.input.display()
            ),
            Ok(false) => eprintln!("[tiled-to-shapes] No scaffolding needed"),
            Err(e) => {
                eprintln!("error scaffolding TSX: {e}");
                std::process::exit(1);
            }
        }
    }

    if args.dry_run {
        // Parse and validate only
        match parse_tsx(&args.input) {
            Ok(t) => {
                eprintln!(
                    "[tiled-to-shapes] Dry run OK: {} Wang set(s), {}×{} tiles, {} columns",
                    t.wang_sets.len(),
                    t.tile_width,
                    t.tile_height,
                    t.columns
                );
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    }

    // Full conversion
    let config = default_convert_config();
    let tileset = match convert_tileset(&args.input, &config) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error converting tileset: {e}");
            std::process::exit(1);
        }
    };

    let tsx_filename = args
        .input
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.tsx");

    let code = generate_tileset_code(&tileset, &args.fn_name, tsx_filename);

    match args.output {
        None => {
            print!("{code}");
        }
        Some(dir) => {
            let out_file = dir.join(format!("{}.rs", args.fn_name));
            if let Err(e) = std::fs::create_dir_all(&dir) {
                eprintln!("error creating output directory: {e}");
                std::process::exit(1);
            }
            if let Err(e) = std::fs::write(&out_file, &code) {
                eprintln!("error writing output: {e}");
                std::process::exit(1);
            }
            eprintln!("[tiled-to-shapes] Written to {}", out_file.display());
        }
    }
}
