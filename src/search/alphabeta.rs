
struct SearchStack{
    lmr: bool,
    nullmove: bool,
    futility: bool,
    razor: bool,
    ispv: bool,
    reductions: u8, // half-ply reductions
    ply: u16,
}