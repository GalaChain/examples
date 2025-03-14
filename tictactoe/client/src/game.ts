import { Game, Move } from 'boardgame.io';

export interface TicTacContractState {
  dto?: unknown | undefined;
  cells: (string | null)[];
  winner: string | null;
}

const makeMove: Move<TicTacContractState> = ({ G, ctx }, id: number, dto?: string) => {
  if (G.cells[id] !== null) return;
  G.dto = dto;
  G.cells[id] = ctx.currentPlayer;
};

export const TicTacContract: Game<TicTacContractState> = {
  name: 'tic-tac-contract',
  setup: ({ ctx, ...plugins }, setupData: TicTacContractState) => ({
    dto: setupData.dto,
    cells: Array(9).fill(null),
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
      if (G.cells[a] !== null && G.cells[a] === G.cells[b] && G.cells[a] === G.cells[c]) {
        return { winner: ctx.currentPlayer };
      }
    }

    if (G.cells.every(cell => cell !== null)) {
      return { draw: true };
    }
  },
};
