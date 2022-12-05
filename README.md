# Chess AI

A fully rust chess engine + AI + GUI written in Rust and Macroquad (a graphics library)

AI is a minimax search with alpha-beta pruning, move-ordering, and Tomasz Michniewski's simplified evaluation function

- Release hosted at <https://chess.jamesalin.com>
- Docs here <https://chess.jamesalin.com/docs> (see documented source code)

## Building

Clone and build using `cargo build`

**Make sure you copy `/assets` from this repo and put it in base directory, else rust will panic!**

## How the AI works

The AI uses a minimax algorithm to calculate the next best move of a chess game. The algorithm works by trying to minimize your opponents score, while maximizing your own score, thus the minimax name.

In this chess game, it does this by first going through every possible move (sorted by a given heuristic to estimate how good a move will be), and simulating said move, storing the new scores for the player and agent. After each move is simulated, a new minimax search will begin, and TODO
