/**
 * @fileoverview Tests for the setMatchState function
 */

import { GalaChainContext } from "@gala-chain/chaincode";
import { createValidChainObject, ValidationFailedError } from "@gala-chain/api";
import { TestChaincode } from "@gala-chain/test";

import { setMatchState } from "../setMatchState";
import { createMatch } from "../createMatch";
import { TicTacMatch } from "../TicTacMatch";
import { CreateMatchDto, MatchStateDto } from "../dtos";
import { PlayerSymbol, GameStatus } from "../types";

describe("setMatchState", () => {
  let ctx: GalaChainContext;
  const matchID = "state-test-match";

  beforeEach(async () => {
    ctx = new TestChaincode([TicTacMatch]).ctx();
    
    // Create initial match for testing
    const createDto = await createValidChainObject(CreateMatchDto, {
      matchID,
      initialStateID: `${matchID}-initial`,
      state: {
        _stateID: 0,
        G: { 
          board: Array(9).fill(null),
          currentPlayer: PlayerSymbol.X 
        },
        ctx: { 
          numPlayers: 2, 
          turn: 1, 
          currentPlayer: "0", 
          phase: "play" 
        },
        plugins: {}
      },
      metadata: {
        gameName: "tic-tac-toe",
        players: {
          "0": { id: "0", name: "Player X" },
          "1": { id: "1", name: "Player O" }
        },
        setupData: {},
        gameover: undefined,
        nextMatchID: undefined,
        unlisted: false
      },
      uniqueKey: `${matchID}-${Date.now()}`
    });

    await createMatch(ctx, createDto);
  });

  it("should update match state successfully", async () => {
    const newBoard = Array(9).fill(null);
    newBoard[0] = PlayerSymbol.X; // First move

    const stateDto = await createValidChainObject(MatchStateDto, {
      matchID,
      state: {
        _stateID: 1,
        G: { 
          board: newBoard,
          currentPlayer: PlayerSymbol.O 
        },
        ctx: { 
          numPlayers: 2, 
          turn: 2, 
          currentPlayer: "1", 
          phase: "play" 
        },
        plugins: {}
      },
      metadata: {
        gameName: "tic-tac-toe",
        players: {
          "0": { id: "0", name: "Player X" },
          "1": { id: "1", name: "Player O" }
        },
        setupData: {},
        gameover: undefined,
        nextMatchID: undefined,
        unlisted: false
      },
      deltalog: []
    });

    const result = await setMatchState(ctx, stateDto);

    expect(result).toBeInstanceOf(TicTacMatch);
    expect(result.matchID).toBe(matchID);
    expect(result.board).toEqual(newBoard);
    expect(result.status).toBe(GameStatus.IN_PROGRESS);
  });

  it("should handle game completion with winner", async () => {
    // Create winning board for Player X (top row)
    const winningBoard = [
      PlayerSymbol.X, PlayerSymbol.X, PlayerSymbol.X,
      PlayerSymbol.O, PlayerSymbol.O, null,
      null, null, null
    ];

    const stateDto = await createValidChainObject(MatchStateDto, {
      matchID,
      state: {
        _stateID: 5,
        G: { 
          board: winningBoard,
          currentPlayer: PlayerSymbol.O 
        },
        ctx: { 
          numPlayers: 2, 
          turn: 6, 
          currentPlayer: "1", 
          phase: "play",
          gameover: { winner: "0" }
        },
        plugins: {}
      },
      metadata: {
        gameName: "tic-tac-toe",
        players: {
          "0": { id: "0", name: "Player X" },
          "1": { id: "1", name: "Player O" }
        },
        setupData: {},
        gameover: { winner: "0" },
        nextMatchID: undefined,
        unlisted: false
      },
      deltalog: []
    });

    const result = await setMatchState(ctx, stateDto);

    expect(result.status).toBe(GameStatus.FINISHED);
    expect(result.board).toEqual(winningBoard);
  });

  it("should handle draw game", async () => {
    // Create board with draw state
    const drawBoard = [
      PlayerSymbol.X, PlayerSymbol.O, PlayerSymbol.X,
      PlayerSymbol.O, PlayerSymbol.O, PlayerSymbol.X,
      PlayerSymbol.O, PlayerSymbol.X, PlayerSymbol.O
    ];

    const stateDto = await createValidChainObject(MatchStateDto, {
      matchID,
      state: {
        _stateID: 9,
        G: { 
          board: drawBoard,
          currentPlayer: PlayerSymbol.X 
        },
        ctx: { 
          numPlayers: 2, 
          turn: 10, 
          currentPlayer: "0", 
          phase: "play",
          gameover: { draw: true }
        },
        plugins: {}
      },
      metadata: {
        gameName: "tic-tac-toe",
        players: {
          "0": { id: "0", name: "Player X" },
          "1": { id: "1", name: "Player O" }
        },
        setupData: {},
        gameover: { draw: true },
        nextMatchID: undefined,
        unlisted: false
      },
      deltalog: []
    });

    const result = await setMatchState(ctx, stateDto);

    expect(result.status).toBe(GameStatus.FINISHED);
    expect(result.board).toEqual(drawBoard);
  });

  it("should reject update for non-existent match", async () => {
    const stateDto = await createValidChainObject(MatchStateDto, {
      matchID: "non-existent-match",
      state: {
        _stateID: 1,
        G: { 
          board: Array(9).fill(null),
          currentPlayer: PlayerSymbol.X 
        },
        ctx: { 
          numPlayers: 2, 
          turn: 1, 
          currentPlayer: "0", 
          phase: "play" 
        },
        plugins: {}
      },
      metadata: {
        gameName: "tic-tac-toe",
        players: {},
        setupData: {},
        gameover: undefined,
        nextMatchID: undefined,
        unlisted: false
      },
      deltalog: []
    });

    await expect(setMatchState(ctx, stateDto))
      .rejects.toThrow();
  });

  it("should reject update with no state and no deltalog", async () => {
    const invalidDto = await createValidChainObject(MatchStateDto, {
      matchID,
      state: undefined as any,
      metadata: {
        gameName: "tic-tac-toe",
        players: {},
        setupData: {},
        gameover: undefined,
        nextMatchID: undefined,
        unlisted: false
      },
      deltalog: undefined as any
    });

    await expect(setMatchState(ctx, invalidDto))
      .rejects.toThrow(ValidationFailedError);
  });

  it("should update match with deltalog only", async () => {
    const deltalog = [
      {
        action: { type: "MAKE_MOVE", payload: [0] },
        _stateID: 0,
        turn: 1,
        phase: "play"
      }
    ];

    const stateDto = await createValidChainObject(MatchStateDto, {
      matchID,
      state: undefined as any,
      metadata: {
        gameName: "tic-tac-toe",
        players: {
          "0": { id: "0", name: "Player X" },
          "1": { id: "1", name: "Player O" }
        },
        setupData: {},
        gameover: undefined,
        nextMatchID: undefined,
        unlisted: false
      },
      deltalog
    });

    const result = await setMatchState(ctx, stateDto);

    expect(result).toBeInstanceOf(TicTacMatch);
    expect(result.matchID).toBe(matchID);
  });

  it("should preserve existing match data when updating", async () => {
    const originalPlayerX = "original-player-x";
    const originalPlayerO = "original-player-o";

    // First, update the match to have known players
    const setupDto = await createValidChainObject(MatchStateDto, {
      matchID,
      state: {
        _stateID: 0,
        G: { 
          board: Array(9).fill(null),
          currentPlayer: PlayerSymbol.X 
        },
        ctx: { 
          numPlayers: 2, 
          turn: 1, 
          currentPlayer: "0", 
          phase: "play" 
        },
        plugins: {}
      },
      metadata: {
        gameName: "tic-tac-toe",
        players: {
          "0": { id: "0", name: "Player X" },
          "1": { id: "1", name: "Player O" }
        },
        setupData: {},
        gameover: undefined,
        nextMatchID: undefined,
        unlisted: false
      },
      deltalog: []
    });

    await setMatchState(ctx, setupDto);

    // Now update just the board state
    const newBoard = Array(9).fill(null);
    newBoard[4] = PlayerSymbol.X; // Center move

    const updateDto = await createValidChainObject(MatchStateDto, {
      matchID,
      state: {
        _stateID: 1,
        G: { 
          board: newBoard,
          currentPlayer: PlayerSymbol.O 
        },
        ctx: { 
          numPlayers: 2, 
          turn: 2, 
          currentPlayer: "1", 
          phase: "play" 
        },
        plugins: {}
      },
      metadata: {
        gameName: "tic-tac-toe",
        players: {
          "0": { id: "0", name: "Player X" },
          "1": { id: "1", name: "Player O" }
        },
        setupData: {},
        gameover: undefined,
        nextMatchID: undefined,
        unlisted: false
      },
      deltalog: []
    });

    const result = await setMatchState(ctx, updateDto);

    expect(result.board[4]).toBe(PlayerSymbol.X);
    expect(result.matchID).toBe(matchID);
  });
});