/*
 * Copyright (c) Gala Games Inc. All rights reserved.
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */
import { GalaChainResponse, GalaChainResponseType, randomUniqueKey } from "@gala-chain/api";
import { ChainClient, ChainUser, CommonContractAPI, commonContractAPI } from "@gala-chain/client";
import { AdminChainClients, TestClients, transactionErrorKey, transactionSuccess } from "@gala-chain/test";
import { afterAll, beforeAll, describe, expect, jest, test } from "@jest/globals";

import {
  CreateMatchDto,
  FetchMatchesDto,
  FetchMatchesResDto,
  JoinMatchDto,
  MakeMoveDto,
  TicTacMatch
} from "../src/tictac";
import { GameStatus, PlayerSymbol } from "../src/tictac/types";

jest.setTimeout(30000);

describe("TicTac Contract", () => {
  // Initial boardgame.io state for a new game
  const initialBoardgameState = JSON.stringify({
    cells: Array(9).fill(null),
    currentPlayer: "X",
    winner: null
  });
  const tictacContractConfig = {
    tictac: {
      channel: "product-channel",
      chaincode: "basic-product",
      contract: "TicTacContract",
      api: tictacContractAPI
    }
  };
  let client: AdminChainClients<typeof tictacContractConfig>;
  let playerX: ChainUser;
  let playerO: ChainUser;
  let matchId: string;
  let matchCreatedAt: number;

  beforeAll(async () => {
    client = await TestClients.createForAdmin(tictacContractConfig);
    playerX = await client.createRegisteredUser();
    playerO = await client.createRegisteredUser();
  });

  afterAll(async () => {
    await client.disconnect();
  });

  test("Create a new game", async () => {
    // Given
    const dto = new CreateMatchDto(
      playerX.identityKey,
      undefined,
      randomUniqueKey(),
      initialBoardgameState
    ).signed(playerX.privateKey);

    // When
    const response = await client.tictac.CreateMatch(dto);

    // Then
    expect(response).toEqual(transactionSuccess());

    // Store game ID for subsequent tests
    const fetchDto = new FetchMatchesDto().signed(playerX.privateKey);
    const fetchResponse = await client.tictac.FetchMatches(fetchDto);
    const pagedGames = (fetchResponse as GalaChainResponse<FetchMatchesResDto>).Data!;
    expect(pagedGames).toBeDefined();
    expect(pagedGames.results.length).toBeGreaterThan(0);

    // Find our newly created game
    const ourGame = pagedGames.results.find(
      (game) => game.playerX === playerX.identityKey && game.playerO === undefined
    );
    expect(ourGame).toBeDefined();
    expect(ourGame).toEqual(
      expect.objectContaining({
        playerX: playerX.identityKey,
        status: GameStatus.IN_PROGRESS,
        createdAt: expect.any(Number),
        currentPlayer: PlayerSymbol.X,
        board: Array(9).fill(null),
        boardgameState: initialBoardgameState,
        matchId: expect.any(String),
        lastMoveAt: expect.any(Number)
      })
    );
    matchId = ourGame!.matchId;
    matchCreatedAt = ourGame!.createdAt;
  });

  test("Fetch games for player X", async () => {
    // Given
    const dto = new FetchMatchesDto().signed(playerX.privateKey);

    // When
    const response = await client.tictac.FetchMatches(dto);

    // Then
    const pagedGames = (response as GalaChainResponse<FetchMatchesResDto>).Data!;
    expect(pagedGames).toBeDefined();
    expect(pagedGames.results.length).toBeGreaterThan(0);

    // Find our game
    const ourGame = pagedGames.results.find((game) => game.playerX === playerX.identityKey);
    expect(ourGame).toBeDefined();
    expect(ourGame!).toEqual(
      expect.objectContaining({
        playerX: playerX.identityKey,
        status: GameStatus.IN_PROGRESS,
        currentPlayer: PlayerSymbol.X,
        board: Array(9).fill(null)
      })
    );
    expect(ourGame?.boardgameState).toBeDefined();
  });

  test("Allow player O to join open game", async () => {
    // Given
    const dto = new JoinMatchDto(matchId, undefined, playerO.identityKey, randomUniqueKey()).signed(
      playerO.privateKey
    );

    const expectedMatch = new TicTacMatch(
      matchId,
      playerX.identityKey,
      playerO.identityKey,
      matchCreatedAt,
      initialBoardgameState
    );

    // When
    const response = await client.tictac.JoinMatch(dto);

    // Then
    expect(response).toEqual(transactionSuccess(expectedMatch));
  });

  test("Make valid moves alternating between players", async () => {
    // Player X makes first move
    const state1 = JSON.stringify({
      cells: [PlayerSymbol.X, null, null, null, null, null, null, null, null],
      currentPlayer: "O",
      winner: null
    });
    const moveX1 = new MakeMoveDto(matchId, 0, randomUniqueKey(), state1).signed(playerX.privateKey);
    const responseX1 = await client.tictac.MakeMove(moveX1);
    expect(responseX1).toEqual(transactionSuccess());

    // Player O makes second move
    const state2 = JSON.stringify({
      cells: [PlayerSymbol.X, null, null, null, PlayerSymbol.O, null, null, null, null],
      currentPlayer: "X",
      winner: null
    });
    const moveO1 = new MakeMoveDto(matchId, 4, randomUniqueKey(), state2).signed(playerO.privateKey);
    const responseO1 = await client.tictac.MakeMove(moveO1);
    expect(responseO1).toEqual(transactionSuccess());

    // Verify game state
    const dto = new FetchMatchesDto(undefined, matchId).signed(playerX.privateKey);
    const response = await client.tictac.FetchMatches(dto);
    const pagedGames = (response as GalaChainResponse<FetchMatchesResDto>).Data!;
    expect(pagedGames).toBeDefined();
    expect(pagedGames.results.length).toBeGreaterThan(0);
    const game = pagedGames.results[0];

    expect(game.board[0]).toBe(PlayerSymbol.X);
    expect(game.board[4]).toBe(PlayerSymbol.O);
    expect(game.currentPlayer).toBe(PlayerSymbol.X);
    expect(game.status).toBe(GameStatus.IN_PROGRESS);
    expect(game.boardgameState).toBe(state2);
  });

  test("Fail to make move out of turn", async () => {
    // Given - Player O tries to move when it's X's turn
    const dto = new MakeMoveDto(matchId, 1, randomUniqueKey()).signed(playerO.privateKey);

    // When
    const response = await client.tictac.MakeMove(dto);

    // Then
    expect(response).toEqual(transactionErrorKey("INVALID_MOVE"));
  });

  test("Fail to make move in occupied position", async () => {
    // Given - Player X tries to move to an occupied position
    const dto = new MakeMoveDto(matchId, 0, randomUniqueKey()).signed(playerX.privateKey);

    // When
    const response = await client.tictac.MakeMove(dto);

    // Then
    expect(response).toEqual(transactionErrorKey("INVALID_MOVE"));
  });

  test("Complete game with X winning", async () => {
    // X plays center-right
    const state3 = JSON.stringify({
      cells: [PlayerSymbol.X, PlayerSymbol.X, null, null, PlayerSymbol.O, null, null, null, null],
      currentPlayer: "O",
      winner: null
    });
    const moveX2 = new MakeMoveDto(matchId, 1, randomUniqueKey(), state3).signed(playerX.privateKey);
    await client.tictac.MakeMove(moveX2);

    // O plays bottom-center
    const state4 = JSON.stringify({
      cells: [PlayerSymbol.X, PlayerSymbol.X, null, null, PlayerSymbol.O, null, null, PlayerSymbol.O, null],
      currentPlayer: "X",
      winner: null
    });
    const moveO2 = new MakeMoveDto(matchId, 7, randomUniqueKey(), state4).signed(playerO.privateKey);
    await client.tictac.MakeMove(moveO2);

    // X completes the win with top-right
    const state5 = JSON.stringify({
      cells: [
        PlayerSymbol.X,
        PlayerSymbol.X,
        PlayerSymbol.X,
        null,
        PlayerSymbol.O,
        null,
        null,
        PlayerSymbol.O,
        null
      ],
      currentPlayer: "O",
      winner: "X"
    });
    const moveX3 = new MakeMoveDto(matchId, 2, randomUniqueKey(), state5).signed(playerX.privateKey);
    const responseX3 = await client.tictac.MakeMove(moveX3);
    expect(responseX3).toEqual(transactionSuccess());

    // Verify game state
    const dto = new FetchMatchesDto(undefined, matchId).signed(playerX.privateKey);
    const response = await client.tictac.FetchMatches(dto);
    const pagedGames = (response as GalaChainResponse<FetchMatchesResDto>).Data!;
    expect(pagedGames).toBeDefined();
    expect(pagedGames.results.length).toBeGreaterThan(0);
    const game = pagedGames.results[0];

    expect(game.status).toBe(GameStatus.X_WON);
    expect(game.board[0]).toBe(PlayerSymbol.X);
    expect(game.board[1]).toBe(PlayerSymbol.X);
    expect(game.board[2]).toBe(PlayerSymbol.X);
    expect(game.boardgameState).toBe(state5);
  });
});

interface TicTacContractAPI {
  CreateMatch(dto: CreateMatchDto): Promise<GalaChainResponse<void>>;
  JoinMatch(dto: JoinMatchDto): Promise<GalaChainResponse<TicTacMatch>>;
  MakeMove(dto: MakeMoveDto): Promise<GalaChainResponse<void>>;
  FetchMatches(dto: FetchMatchesDto): Promise<GalaChainResponse<FetchMatchesResDto>>;
}

function tictacContractAPI(client: ChainClient): TicTacContractAPI & CommonContractAPI {
  return {
    ...commonContractAPI(client),

    CreateMatch(dto: CreateMatchDto) {
      return client.submitTransaction("CreateMatch", dto) as Promise<GalaChainResponse<void>>;
    },

    JoinMatch(dto: JoinMatchDto) {
      return client.submitTransaction("JoinMatch", dto) as Promise<GalaChainResponse<TicTacMatch>>;
    },

    MakeMove(dto: MakeMoveDto) {
      return client.submitTransaction("MakeMove", dto) as Promise<GalaChainResponse<void>>;
    },

    FetchMatches(dto: FetchMatchesDto) {
      return client.evaluateTransaction("FetchMatches", dto) as Promise<
        GalaChainResponse<FetchMatchesResDto>
      >;
    }
  };
}
