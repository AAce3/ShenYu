# Shen Yu
A UCI chess engine written from scratch in rust. It is an original work, but takes inspiration from many other engines.
Shen Yu is currently in active development.

It has a [CCRL](https://ccrl.chessdom.com/ccrl/) rating of 2508. (Thanks to Gabor Szots for testing!) 
## User Interface
Shen Yu does not come with a user interface. It requires a user interface that implements UCI to run. Popular user interfaces include:

- [Arena](http://www.playwitharena.de/)
- [CuteChess](https://cutechess.com/)
- [Banksia](https://banksiagui.com/)

and many others. 

## Features
### Move Generation:
  - Fancy Magic Bitboards
  - fully legal move generation
  - Staged Move generation (TTMove, Captures, Killers, Losing captures, Quiets)
### Evaluation:
  - Tapered PeSTO PSQTs
### Search:
  - Iterative Deepening
  - Alpha-Beta in a Negamax framework
  - PVS
  - Transposition Table
  - MVV/LVA move ordering
  - SEE move ordering for losing captures
  - Killer heuristic
  - History heuristic
  - Quiescience Search
  - SEE pruning in quiescience search
  - Null Move Pruning (2.0.0+)
  - Late Move Reductions (2.0.0+)
  - Futility Pruning (2.0.0+)
  - Late Move Pruning (2.0.0+)
## Building and Compiling
Shen Yu only comes with binaries for windows and linux. To compile, install [Rust](https://www.rust-lang.org/tools/install) and clone the repository.
Navigate to the project, and use
```
cargo build --release
```
to generate a compiled binary.
In the 'target' folder, a folder named 'release' should show up. The executable can be found in that folder, titled "ShenYu"
