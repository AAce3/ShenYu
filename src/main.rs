use uci::gameloop;

mod eval;
mod movegen;
pub mod search;
mod uci;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "2");
    gameloop();
}


