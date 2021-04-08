const HELP: &str = "\
Usage: crabmail

FLAGS:
  -h, --help            Prints this help information and exits.
";

fn main() {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }
}
