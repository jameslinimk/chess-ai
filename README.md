# Chess AI

A fully rust chess engine + AI + GUI written in Rust and Macroquad (a graphics library)

AI is a minimax search with alpha-beta pruning, transposition table, move-ordering, and Tomasz Michniewski's simplified evaluation function

- Release hosted at <https://chess.jamesalin.com>
- Docs here <https://chess.jamesalin.com/docs>

## How the AI works

The chess AI uses the minimax algorithm with multiple other techniques to calculate the best move.

The minimax algorithm does this by simulating all possible moves and their resulting positions, and choosing the move that leads to the best outcome for the current player. (here, it is the sum of the player pieces - sum of the enemy pieces, with other things such as castling and position in effect) The algorithm uses a recursive function to simulate each possible move and calculate the best response to that move by the opponent. This process continues until a terminal position is reached, at which point the algorithm can evaluate the position to determine its value for the current player.

Alpha-beta pruning is a technique used to improve the efficiency of the minimax algorithm by reducing the number of positions that need to be evaluated. It works by keeping track of the best move found so far for the current player (alpha) and the worst move found so far for the opponent (beta). If at any point during the search the current alpha value is greater than or equal to the beta value, the search can be stopped because it is not possible for the current player to achieve a better outcome than the worst outcome for the opponent.

A transposition table is a data structure used to store information about previously evaluated positions. When the minimax algorithm is analyzing a position, it can check the transposition table to see if the position has already been evaluated. If it has, the algorithm can use the previously calculated value instead of performing the calculation again, which can save a lot of time and computational resources.

Sorted moves is a technique used to improve the efficiency of the minimax algorithm by ordering the list of possible moves in a way that allows the algorithm to search the most promising moves first. This can help the algorithm find a good move more quickly, because it is more likely to reach a terminal position and evaluate the position sooner. Here, the move is sorted by the value of the piece being moved, and the value of the piece being captured, the position of the new piece as well as capturing pieces.
