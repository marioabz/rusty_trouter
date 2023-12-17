use clap::Parser;
mod tracer;

#[derive(Parser, Debug)]
struct Cli {
    host: String,
    packet_size: u8,
}

fn main() {
    let args = Cli::parse();
    tracer::run_tracerouter(&args.host);
}
