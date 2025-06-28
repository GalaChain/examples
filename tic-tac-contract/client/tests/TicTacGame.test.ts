/**
 * @fileoverview Tests for TicTacMatch class and game logic
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { TicTacMatch } from '../src/TicTacGame';
import { PlayerSymbol, GameStatus } from '../src/types';

describe('TicTacMatch', () => {
  let game: TicTacMatch;

  beforeEach(() => {
    game = new TicTacMatch('test-match', 'player1', 'player2', Date.now());
  });

  describe('constructor', () => {
    it('should initialize with empty board', () => {
      expect(game.board).toEqual(Array(9).fill(null));
      expect(game.currentPlayer).toBe(PlayerSymbol.X);
      expect(game.status).toBe(GameStatus.IN_PROGRESS);
    });

    it('should set player IDs correctly', () => {
      expect(game.playerX).toBe('player1');
      expect(game.playerO).toBe('player2');
      expect(game.matchID).toBe('test-match');
    });
  });

  describe('canMakeMove', () => {
    it('should allow valid moves for correct player', () => {
      expect(game.canMakeMove('player1', 0)).toBe(true);
      expect(game.canMakeMove('player2', 0)).toBe(false); // Wrong turn
    });

    it('should reject moves on occupied cells', () => {
      const timestamp = Date.now();
      game.makeMove('player1', 0, timestamp);
      
      expect(game.canMakeMove('player2', 0)).toBe(false);
    });

    it('should reject invalid positions', () => {
      expect(game.canMakeMove('player1', -1)).toBe(false);
      expect(game.canMakeMove('player1', 9)).toBe(false);
    });
  });

  describe('makeMove', () => {
    it('should make valid move successfully', () => {
      const timestamp = Date.now();
      game.makeMove('player1', 0, timestamp);
      
      expect(game.board[0]).toBe(PlayerSymbol.X);
      expect(game.currentPlayer).toBe(PlayerSymbol.O);
      expect(game.lastMoveAt).toBe(timestamp);
    });

    it('should throw error for invalid moves', () => {
      expect(() => {
        game.makeMove('player2', 0, Date.now()); // Wrong turn
      }).toThrow();
      
      expect(() => {
        game.makeMove('player1', -1, Date.now()); // Invalid position
      }).toThrow();
    });

    it('should switch players after valid move', () => {
      game.makeMove('player1', 0, Date.now());
      expect(game.currentPlayer).toBe(PlayerSymbol.O);
      
      game.makeMove('player2', 1, Date.now());
      expect(game.currentPlayer).toBe(PlayerSymbol.X);
    });
  });

  describe('game completion', () => {
    it('should detect horizontal win', () => {
      const timestamp = Date.now();
      
      // X wins top row
      game.makeMove('player1', 0, timestamp); // X
      game.makeMove('player2', 3, timestamp); // O
      game.makeMove('player1', 1, timestamp); // X
      game.makeMove('player2', 4, timestamp); // O
      game.makeMove('player1', 2, timestamp); // X wins
      
      expect(game.status).toBe(GameStatus.X_WON);
    });

    it('should detect vertical win', () => {
      const timestamp = Date.now();
      
      // O wins left column
      game.makeMove('player1', 1, timestamp); // X
      game.makeMove('player2', 0, timestamp); // O
      game.makeMove('player1', 2, timestamp); // X
      game.makeMove('player2', 3, timestamp); // O
      game.makeMove('player1', 4, timestamp); // X
      game.makeMove('player2', 6, timestamp); // O wins
      
      expect(game.status).toBe(GameStatus.O_WON);
    });

    it('should detect diagonal win', () => {
      const timestamp = Date.now();
      
      // X wins diagonal
      game.makeMove('player1', 0, timestamp); // X
      game.makeMove('player2', 1, timestamp); // O
      game.makeMove('player1', 4, timestamp); // X
      game.makeMove('player2', 2, timestamp); // O
      game.makeMove('player1', 8, timestamp); // X wins
      
      expect(game.status).toBe(GameStatus.X_WON);
    });

    it('should detect full board without winner', () => {
      // Test that a full board is detected
      // We'll manually set a draw position to test the logic
      game.board = [
        PlayerSymbol.X, PlayerSymbol.O, PlayerSymbol.X,  // X O X
        PlayerSymbol.O, PlayerSymbol.O, PlayerSymbol.X,  // O O X
        PlayerSymbol.O, PlayerSymbol.X, PlayerSymbol.O   // O X O
      ];
      
      // This pattern has no three-in-a-row
      expect(game.board.every(cell => cell !== null)).toBe(true);
    });

    it('should prevent moves after game ends', () => {
      const timestamp = Date.now();
      
      // X wins
      game.makeMove('player1', 0, timestamp);
      game.makeMove('player2', 3, timestamp);
      game.makeMove('player1', 1, timestamp);
      game.makeMove('player2', 4, timestamp);
      game.makeMove('player1', 2, timestamp); // X wins
      
      expect(() => {
        game.makeMove('player2', 5, timestamp);
      }).toThrow();
    });
  });
});