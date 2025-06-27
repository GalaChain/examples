/**
 * @fileoverview Unit tests for TicTacContract class and its core methods
 */

import { GalaChainContext } from "@gala-chain/chaincode";
import { createValidChainObject } from "@gala-chain/api";
import { TestChaincode } from "@gala-chain/test";

import { TicTacContract } from "../TicTacContract";
import { TicTacMatch } from "../TicTacMatch";
import { CreateMatchDto, FetchMatchDto, MatchStateDto, JoinMatchDto } from "../dtos";
import { GameStatus, PlayerSymbol } from "../types";

describe("TicTacContract", () => {
  let contract: TicTacContract;
  let ctx: GalaChainContext;

  beforeEach(async () => {
    contract = new TicTacContract();
    ctx = new TestChaincode([TicTacMatch]).ctx();
  });

  describe("CreateMatch", () => {
    it("should create a new match successfully", async () => {
      const matchID = "test-match-123";
      const createDto = await createValidChainObject(CreateMatchDto, {
        matchID,
        initialStateID: `${matchID}-initial`,
        state: {
          _stateID: 0,
          G: { board: Array(9).fill(null), currentPlayer: PlayerSymbol.X },
          ctx: { numPlayers: 2, turn: 1, currentPlayer: "0", phase: "play" },
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

      const result = await contract.CreateMatch(ctx, createDto);

      expect(result).toEqual(createDto);
      
      // Verify match was created in state
      const matchKey = TicTacMatch.getCompositeKeyFromParts(TicTacMatch.INDEX_KEY, [matchID]);
      const storedMatch = await ctx.stub.getState(matchKey);
      expect(storedMatch).toBeDefined();
    });

    it("should reject duplicate match creation", async () => {
      const matchID = "duplicate-match";
      const createDto = await createValidChainObject(CreateMatchDto, {
        matchID,
        initialStateID: `${matchID}-initial`,
        state: {
          _stateID: 0,
          G: { board: Array(9).fill(null), currentPlayer: PlayerSymbol.X },
          ctx: { numPlayers: 2, turn: 1, currentPlayer: "0", phase: "play" },
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
        uniqueKey: `${matchID}-${Date.now()}`
      });

      // Create match first time
      await contract.CreateMatch(ctx, createDto);

      // Attempt to create again with different unique key
      const duplicateDto = { 
        ...createDto, 
        uniqueKey: `${matchID}-${Date.now() + 1000}` 
      };

      await expect(contract.CreateMatch(ctx, duplicateDto))
        .rejects.toThrow();
    });
  });

  describe("FetchMatch", () => {
    beforeEach(async () => {
      // Create a test match
      const matchID = "fetch-test-match";
      const createDto = await createValidChainObject(CreateMatchDto, {
        matchID,
        initialStateID: `${matchID}-initial`,
        state: {
          _stateID: 0,
          G: { board: Array(9).fill(null), currentPlayer: PlayerSymbol.X },
          ctx: { numPlayers: 2, turn: 1, currentPlayer: "0", phase: "play" },
          plugins: {}
        },
        metadata: {
          gameName: "tic-tac-toe",
          players: {
            "0": { id: "0", name: "Player X" }
          },
          setupData: {},
          gameover: undefined,
          nextMatchID: undefined,
          unlisted: false
        },
        uniqueKey: `${matchID}-${Date.now()}`
      });

      await contract.CreateMatch(ctx, createDto);
    });

    it("should fetch existing match successfully", async () => {
      const fetchDto = await createValidChainObject(FetchMatchDto, {
        matchID: "fetch-test-match"
      });

      const result = await contract.FetchMatch(ctx, fetchDto);

      expect(result).toBeDefined();
      expect(result.matchID).toBe("fetch-test-match");
      expect(result.state).toBeDefined();
      expect(result.metadata).toBeDefined();
    });

    it("should throw error for non-existent match", async () => {
      const fetchDto = await createValidChainObject(FetchMatchDto, {
        matchID: "non-existent-match"
      });

      await expect(contract.FetchMatch(ctx, fetchDto))
        .rejects.toThrow();
    });
  });

  describe("SetMatchState", () => {
    const matchID = "state-test-match";

    beforeEach(async () => {
      // Create a test match first
      const createDto = await createValidChainObject(CreateMatchDto, {
        matchID,
        initialStateID: `${matchID}-initial`,
        state: {
          _stateID: 0,
          G: { board: Array(9).fill(null), currentPlayer: PlayerSymbol.X },
          ctx: { numPlayers: 2, turn: 1, currentPlayer: "0", phase: "play" },
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

      await contract.CreateMatch(ctx, createDto);
    });

    it("should update match state successfully", async () => {
      const newBoard = Array(9).fill(null);
      newBoard[0] = PlayerSymbol.X; // Player X makes first move

      const stateDto = await createValidChainObject(MatchStateDto, {
        matchID,
        state: {
          _stateID: 1,
          G: { board: newBoard, currentPlayer: PlayerSymbol.O },
          ctx: { numPlayers: 2, turn: 2, currentPlayer: "1", phase: "play" },
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

      const result = await contract.SetMatchState(ctx, stateDto);

      expect(result).toBeInstanceOf(TicTacMatch);
      expect(result.matchID).toBe(matchID);
      expect(result.board[0]).toBe(PlayerSymbol.X);
      expect(result.status).toBe(GameStatus.IN_PROGRESS);
    });

    it("should handle game completion", async () => {
      // Create winning board state for X
      const winningBoard = [
        PlayerSymbol.X, PlayerSymbol.X, PlayerSymbol.X,
        PlayerSymbol.O, PlayerSymbol.O, null,
        null, null, null
      ];

      const stateDto = await createValidChainObject(MatchStateDto, {
        matchID,
        state: {
          _stateID: 5,
          G: { board: winningBoard, currentPlayer: PlayerSymbol.O },
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

      const result = await contract.SetMatchState(ctx, stateDto);

      expect(result.status).toBe(GameStatus.FINISHED);
      expect(result.board).toEqual(winningBoard);
    });

    it("should reject state update for non-existent match", async () => {
      const stateDto = await createValidChainObject(MatchStateDto, {
        matchID: "non-existent-match",
        state: {
          _stateID: 1,
          G: { board: Array(9).fill(null), currentPlayer: PlayerSymbol.X },
          ctx: { numPlayers: 2, turn: 1, currentPlayer: "0", phase: "play" },
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

      await expect(contract.SetMatchState(ctx, stateDto))
        .rejects.toThrow();
    });
  });

  describe("JoinMatch", () => {
    const matchID = "join-test-match";

    beforeEach(async () => {
      // Create a match with only one player
      const createDto = await createValidChainObject(CreateMatchDto, {
        matchID,
        initialStateID: `${matchID}-initial`,
        state: {
          _stateID: 0,
          G: { board: Array(9).fill(null), currentPlayer: PlayerSymbol.X },
          ctx: { numPlayers: 2, turn: 1, currentPlayer: "0", phase: "play" },
          plugins: {}
        },
        metadata: {
          gameName: "tic-tac-toe",
          players: {
            "0": { id: "0", name: "Player X" }
          },
          setupData: {},
          gameover: undefined,
          nextMatchID: undefined,
          unlisted: false
        },
        uniqueKey: `${matchID}-${Date.now()}`
      });

      await contract.CreateMatch(ctx, createDto);
    });

    it("should allow player to join match", async () => {
      const joinDto = await createValidChainObject(JoinMatchDto, {
        matchID,
        playerID: "1",
        playerMetadata: {
          id: "1",
          name: "Player O"
        }
      });

      const result = await contract.JoinMatch(ctx, joinDto);

      expect(result).toEqual(joinDto);
      
      // Verify player was added to match metadata
      const fetchDto = await createValidChainObject(FetchMatchDto, { matchID });
      const match = await contract.FetchMatch(ctx, fetchDto);
      expect(match.metadata.players["1"]).toBeDefined();
      expect(match.metadata.players["1"].name).toBe("Player O");
    });

    it("should reject joining non-existent match", async () => {
      const joinDto = await createValidChainObject(JoinMatchDto, {
        matchID: "non-existent-match",
        playerID: "1",
        playerMetadata: {
          id: "1",
          name: "Player O"
        }
      });

      await expect(contract.JoinMatch(ctx, joinDto))
        .rejects.toThrow();
    });
  });
});