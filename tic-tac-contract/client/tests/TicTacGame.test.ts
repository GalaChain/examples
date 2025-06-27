/**
 * @fileoverview Tests for TicTacGame class and game logic
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { TicTacGame } from '../src/TicTacGame';
import { PlayerSymbol, GameStatus } from '../src/types';

describe('TicTacGame', () => {
  let game: TicTacGame;

  beforeEach(() => {
    game = new TicTacGame();
  });

  describe('constructor', () => {
    it('should initialize with empty board', () => {
      expect(game.board).toEqual(Array(9).fill(null));
      expect(game.currentPlayer).toBe(PlayerSymbol.X);
      expect(game.status).toBe(GameStatus.IN_PROGRESS);
    });
  });

  describe('makeMove', () => {
    it('should make valid move successfully', () => {
      const result = game.makeMove(0);
      
      expect(result).toBe(true);
      expect(game.board[0]).toBe(PlayerSymbol.X);
      expect(game.currentPlayer).toBe(PlayerSymbol.O);
    });

    it('should reject move on occupied cell', () => {
      game.makeMove(0); // X takes position 0
      const result = game.makeMove(0); // Try to take same position
      
      expect(result).toBe(false);
      expect(game.board[0]).toBe(PlayerSymbol.X); // Should remain X
      expect(game.currentPlayer).toBe(PlayerSymbol.O); // Should not change turn
    });

    it('should reject move on invalid position', () => {
      const result = game.makeMove(-1);
      expect(result).toBe(false);
      
      const result2 = game.makeMove(9);
      expect(result2).toBe(false);
      
      expect(game.currentPlayer).toBe(PlayerSymbol.X); // Should not change
    });

    it('should reject move when game is finished', () => {
      // Create winning condition for X (top row)
      game.makeMove(0); // X
      game.makeMove(3); // O
      game.makeMove(1); // X
      game.makeMove(4); // O
      game.makeMove(2); // X wins

      expect(game.status).toBe(GameStatus.FINISHED);
      
      // Try to make move after game is finished
      const result = game.makeMove(5);
      expect(result).toBe(false);
    });
  });

  describe('checkWinner', () => {
    it('should detect horizontal win - top row', () => {
      game.board = [
        PlayerSymbol.X, PlayerSymbol.X, PlayerSymbol.X,
        null, null, null,
        null, null, null
      ];
      
      expect(game.checkWinner()).toBe(PlayerSymbol.X);
    });

    it('should detect horizontal win - middle row', () => {
      game.board = [
        null, null, null,
        PlayerSymbol.O, PlayerSymbol.O, PlayerSymbol.O,
        null, null, null
      ];
      
      expect(game.checkWinner()).toBe(PlayerSymbol.O);
    });

    it('should detect horizontal win - bottom row', () => {
      game.board = [
        null, null, null,
        null, null, null,
        PlayerSymbol.X, PlayerSymbol.X, PlayerSymbol.X
      ];
      
      expect(game.checkWinner()).toBe(PlayerSymbol.X);
    });

    it('should detect vertical win - left column', () => {
      game.board = [
        PlayerSymbol.O, null, null,
        PlayerSymbol.O, null, null,
        PlayerSymbol.O, null, null
      ];
      
      expect(game.checkWinner()).toBe(PlayerSymbol.O);
    });

    it('should detect vertical win - middle column', () => {
      game.board = [
        null, PlayerSymbol.X, null,
        null, PlayerSymbol.X, null,
        null, PlayerSymbol.X, null
      ];
      
      expect(game.checkWinner()).toBe(PlayerSymbol.X);
    });

    it('should detect vertical win - right column', () => {
      game.board = [
        null, null, PlayerSymbol.O,
        null, null, PlayerSymbol.O,
        null, null, PlayerSymbol.O
      ];
      
      expect(game.checkWinner()).toBe(PlayerSymbol.O);
    });

    it('should detect diagonal win - top-left to bottom-right', () => {
      game.board = [
        PlayerSymbol.X, null, null,
        null, PlayerSymbol.X, null,
        null, null, PlayerSymbol.X
      ];
      
      expect(game.checkWinner()).toBe(PlayerSymbol.X);
    });

    it('should detect diagonal win - top-right to bottom-left', () => {
      game.board = [
        null, null, PlayerSymbol.O,
        null, PlayerSymbol.O, null,
        PlayerSymbol.O, null, null
      ];
      
      expect(game.checkWinner()).toBe(PlayerSymbol.O);
    });

    it('should return null when no winner', () => {
      game.board = [
        PlayerSymbol.X, PlayerSymbol.O, PlayerSymbol.X,
        PlayerSymbol.O, PlayerSymbol.X, PlayerSymbol.O,
        PlayerSymbol.O, PlayerSymbol.X, null
      ];
      
      expect(game.checkWinner()).toBeNull();
    });
  });

  describe('isDraw', () => {
    it('should detect draw when board is full with no winner', () => {
      game.board = [
        PlayerSymbol.X, PlayerSymbol.O, PlayerSymbol.X,
        PlayerSymbol.O, PlayerSymbol.O, PlayerSymbol.X,
        PlayerSymbol.O, PlayerSymbol.X, PlayerSymbol.O
      ];
      
      expect(game.isDraw()).toBe(true);
    });

    it('should not detect draw when board has empty cells', () => {
      game.board = [
        PlayerSymbol.X, PlayerSymbol.O, PlayerSymbol.X,
        PlayerSymbol.O, PlayerSymbol.O, PlayerSymbol.X,
        PlayerSymbol.O, PlayerSymbol.X, null
      ];
      
      expect(game.isDraw()).toBe(false);
    });

    it('should not detect draw when there is a winner', () => {
      game.board = [
        PlayerSymbol.X, PlayerSymbol.X, PlayerSymbol.X,
        PlayerSymbol.O, PlayerSymbol.O, PlayerSymbol.X,
        PlayerSymbol.O, PlayerSymbol.X, PlayerSymbol.O
      ];
      
      expect(game.isDraw()).toBe(false);
    });
  });

  describe('reset', () => {
    it('should reset game to initial state', () => {
      // Make some moves
      game.makeMove(0);
      game.makeMove(1);
      game.makeMove(2);
      
      expect(game.board[0]).toBe(PlayerSymbol.X);
      expect(game.currentPlayer).toBe(PlayerSymbol.O);
      
      // Reset
      game.reset();
      
      expect(game.board).toEqual(Array(9).fill(null));
      expect(game.currentPlayer).toBe(PlayerSymbol.X);
      expect(game.status).toBe(GameStatus.IN_PROGRESS);
    });
  });

  describe('getValidMoves', () => {
    it('should return all positions when board is empty', () => {
      const validMoves = game.getValidMoves();
      expect(validMoves).toEqual([0, 1, 2, 3, 4, 5, 6, 7, 8]);
    });

    it('should return only empty positions', () => {
      game.makeMove(0); // X
      game.makeMove(4); // O
      game.makeMove(8); // X
      
      const validMoves = game.getValidMoves();
      expect(validMoves).toEqual([1, 2, 3, 5, 6, 7]);
    });

    it('should return empty array when board is full', () => {
      game.board = [
        PlayerSymbol.X, PlayerSymbol.O, PlayerSymbol.X,
        PlayerSymbol.O, PlayerSymbol.O, PlayerSymbol.X,
        PlayerSymbol.O, PlayerSymbol.X, PlayerSymbol.O
      ];
      
      const validMoves = game.getValidMoves();
      expect(validMoves).toEqual([]);
    });
  });

  describe('game flow integration', () => {
    it('should complete full game with X winning', () => {
      // X wins with top row
      expect(game.makeMove(0)).toBe(true); // X
      expect(game.makeMove(3)).toBe(true); // O
      expect(game.makeMove(1)).toBe(true); // X
      expect(game.makeMove(4)).toBe(true); // O
      expect(game.makeMove(2)).toBe(true); // X wins
      
      expect(game.status).toBe(GameStatus.FINISHED);
      expect(game.checkWinner()).toBe(PlayerSymbol.X);
      expect(game.makeMove(5)).toBe(false); // Should not allow more moves
    });

    it('should complete full game with draw', () => {
      // Create draw scenario
      const moves = [4, 0, 8, 2, 6, 3, 1, 7, 5]; // Results in draw
      
      for (let i = 0; i < moves.length; i++) {
        expect(game.makeMove(moves[i])).toBe(true);
      }
      
      expect(game.status).toBe(GameStatus.FINISHED);
      expect(game.checkWinner()).toBeNull();
      expect(game.isDraw()).toBe(true);
    });
  });
});