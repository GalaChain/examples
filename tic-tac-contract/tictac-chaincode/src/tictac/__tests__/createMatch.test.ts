/**
 * @fileoverview Tests for the createMatch function
 */

import { GalaChainContext } from "@gala-chain/chaincode";
import { createValidChainObject } from "@gala-chain/api";
import { TestChaincode } from "@gala-chain/test";

import { createMatch } from "../createMatch";
import { TicTacMatch } from "../TicTacMatch";
import { CreateMatchDto } from "../dtos";
import { PlayerSymbol } from "../types";

describe("createMatch", () => {
  let ctx: GalaChainContext;

  beforeEach(async () => {
    ctx = new TestChaincode([TicTacMatch]).ctx();
  });

  it("should create a new match with valid data", async () => {
    const matchID = "create-test-123";
    const dto = await createValidChainObject(CreateMatchDto, {
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
          "0": { id: "0", name: "Alice" },
          "1": { id: "1", name: "Bob" }
        },
        setupData: {},
        gameover: undefined,
        nextMatchID: undefined,
        unlisted: false
      },
      uniqueKey: `${matchID}-${Date.now()}`
    });

    const result = await createMatch(ctx, dto);

    expect(result).toEqual(dto);
    
    // Verify match metadata was stored
    const matchKey = TicTacMatch.getCompositeKeyFromParts(TicTacMatch.INDEX_KEY, [matchID]);
    const storedData = await ctx.stub.getState(matchKey);
    expect(storedData).toBeDefined();
  });

  it("should store match state and metadata correctly", async () => {
    const matchID = "state-storage-test";
    const testBoard = Array(9).fill(null);
    testBoard[4] = PlayerSymbol.X; // Center position

    const dto = await createValidChainObject(CreateMatchDto, {
      matchID,
      initialStateID: `${matchID}-initial`,
      state: {
        _stateID: 1,
        G: { 
          board: testBoard,
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
          "0": { id: "player-x", name: "Player X" },
          "1": { id: "player-o", name: "Player O" }
        },
        setupData: { variant: "standard" },
        gameover: undefined,
        nextMatchID: undefined,
        unlisted: false
      },
      uniqueKey: `${matchID}-unique-${Date.now()}`
    });

    await createMatch(ctx, dto);

    // Verify the stored match has correct data
    const matchKey = TicTacMatch.getCompositeKeyFromParts(TicTacMatch.INDEX_KEY, [matchID]);
    const storedMatch = await ctx.stub.getState(matchKey);
    expect(storedMatch).toBeDefined();
    
    // Parse and verify stored data structure
    const parsedMatch = JSON.parse(storedMatch.toString());
    expect(parsedMatch.matchID).toBe(matchID);
    expect(parsedMatch.board[4]).toBe(PlayerSymbol.X);
  });

  it("should handle empty player list", async () => {
    const matchID = "empty-players-test";
    const dto = await createValidChainObject(CreateMatchDto, {
      matchID,
      initialStateID: `${matchID}-initial`,
      state: {
        _stateID: 0,
        G: { 
          board: Array(9).fill(null),
          currentPlayer: PlayerSymbol.X 
        },
        ctx: { 
          numPlayers: 0, 
          turn: 1, 
          currentPlayer: "0", 
          phase: "setup" 
        },
        plugins: {}
      },
      metadata: {
        gameName: "tic-tac-toe",
        players: {}, // Empty players object
        setupData: {},
        gameover: undefined,
        nextMatchID: undefined,
        unlisted: true
      },
      uniqueKey: `${matchID}-${Date.now()}`
    });

    const result = await createMatch(ctx, dto);
    expect(result).toEqual(dto);
  });

  it("should properly serialize complex state data", async () => {
    const matchID = "complex-state-test";
    const dto = await createValidChainObject(CreateMatchDto, {
      matchID,
      initialStateID: `${matchID}-initial`,
      state: {
        _stateID: 0,
        G: { 
          board: Array(9).fill(null),
          currentPlayer: PlayerSymbol.X,
          moveHistory: [],
          gameStartTime: Date.now()
        },
        ctx: { 
          numPlayers: 2, 
          turn: 1, 
          currentPlayer: "0", 
          phase: "play",
          activePlayers: ["0", "1"]
        },
        plugins: {
          log: { data: [] },
          events: { data: {} }
        }
      },
      metadata: {
        gameName: "tic-tac-toe",
        players: {
          "0": { 
            id: "0", 
            name: "Advanced Player", 
            credentials: "guest"
          }
        },
        setupData: { 
          timeControl: 300000,
          difficulty: "normal"
        },
        gameover: undefined,
        nextMatchID: undefined,
        unlisted: false
      },
      uniqueKey: `${matchID}-${Date.now()}`
    });

    await expect(createMatch(ctx, dto)).resolves.toEqual(dto);
  });
});