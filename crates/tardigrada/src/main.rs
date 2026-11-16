//! tardigrada — the `tardigrada` binary: a thin shell over the
//! engine lib ([`tardigrada::run`]).

fn main() {
    std::process::exit(tardigrada::run());
}
