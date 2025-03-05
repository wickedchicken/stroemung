use clap::Parser;
use stroemung::window_conf;

#[macroquad::main(window_conf)]
async fn main() {
    let args = stroemung::args::Args::parse();
    stroemung::run(args).await;
}
