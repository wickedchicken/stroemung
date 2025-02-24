use clap::Parser;

fn main() {
    let args = stroemung::args::Args::parse();
    stroemung::run(args);
}
