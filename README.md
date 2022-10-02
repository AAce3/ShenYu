# Shen Yu
A UCI chess engine written from scratch in rust. It is an original work, but takes inspiration from many other engines.
Shen Yu is currently in active development.

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
