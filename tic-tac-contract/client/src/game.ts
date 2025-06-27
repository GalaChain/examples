import { Game, Move } from 'boardgame.io';
import { PlayerSymbol } from './dtos';

/**
 * Game state interface for Tic Tac Toe game
 * 
 * Represents the complete state of a tic-tac-toe game including
 * board position, players, and current move information.
 * 
 * @public
 */
export interface TicTacContractState {
  /** Player X identifier (optional until assigned) */
  playerX?: string | undefined;
  /** Player O identifier (optional until assigned) */
  playerO?: string | undefined;
  /** Index of the last move made on the board (0-8) */
  currentMove: number | null;
  /** 3x3 game board represented as flat array of 9 cells */
  board: (string | null)[];
  /** Winner of the game (player ID) or null if game ongoing */
  winner: string | null;
}

/**
 * Move function for placing a symbol on the board
 * 
 * @param G - Current game state
 * @param ctx - boardgame.io context with player information
 * @param id - Board position index (0-8) where player wants to place symbol
 * @internal
 */
const makeMove: Move<TicTacContractState> = ({ G, ctx }, id: number) => {
  if (G.board[id] !== null) return;

  G.board[id] = ctx.currentPlayer;
  G.currentMove = id;
};

/**
 * boardgame.io Game definition for Tic Tac Toe
 * 
 * Defines the complete game logic including setup, moves, turn structure,
 * and winning conditions for the tic-tac-toe game integrated with GalaChain.
 * 
 * @example
 * ```typescript
 * import { Client } from 'boardgame.io/client';
 * 
 * const client = Client({
 *   game: TicTacContract,
 *   numPlayers: 2
 * });
 * ```
 * 
 * @public
 */
export const TicTacContract: Game<TicTacContractState> = {
  name: 'tic-tac-contract',
  setup: ({ ctx, ...plugins }, setupData: TicTacContractState) => ({
    currentMove: null,
    currentPlayer: PlayerSymbol.X,
    board: Array(9).fill(null),
    winner: null,
  }),

  turn: {
    minMoves: 1,
    maxMoves: 1,
  },

  moves: { makeMove },

  endIf: ({ G, ctx }) => {
    const lines = [
      [0, 1, 2], [3, 4, 5], [6, 7, 8], // rows
      [0, 3, 6], [1, 4, 7], [2, 5, 8], // columns
      [0, 4, 8], [2, 4, 6],            // diagonals
    ];

    for (let line of lines) {
      const [a, b, c] = line;
      if (G.board[a] !== null && G.board[a] === G.board[b] && G.board[a] === G.board[c]) {
        return { winner: ctx.currentPlayer };
      }
    }

    if (G.board.every(cell => cell !== null)) {
      return { draw: true };
    }
  },
};
