use clap::Parser;
mod tracer;

#[derive(Parser, Debug)]
struct Cli {
    host: String,
    packet_size: u8,
}

fn main() {
    let args = Cli::parse();
    let mut trouter = tracer::TRouter::new(&args.host);
    println!("{}", &trouter.init_message());
    trouter.run();
}
