/*
 * Copyright (c) Gala Games Inc. All rights reserved.
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */
import { ChainCallDTO, SubmitCallDTO } from "@gala-chain/api";
import { Type } from "class-transformer";
import { IsArray, IsNumber, IsOptional, IsString, Max, Min, ValidateIf, ValidateNested } from "class-validator";

interface TicTacMatch {
  matchId: string;
  playerX: string;
  playerO: string;
  board: (PlayerSymbol | null)[];
  status: GameStatus;
  currentPlayer: PlayerSymbol;
  createdAt: number;
  lastMoveAt: number;
}

export enum GameStatus {
  IN_PROGRESS = "IN_PROGRESS",
  X_WON = "X_WON",
  O_WON = "O_WON",
  DRAW = "DRAW"
}

export enum PlayerSymbol {
  X = "X",
  O = "O"
}

// below copied/modified from tictac-chaincode/src/dtos.ts
// ref https://grugbrain.dev/

export interface ICreateMatchDto {
  matchId: string;
  playerX?: string;
  playerO?: string;
  boardgameState?: string;
  uniqueKey?: string;
}

export interface IJoinMatchDto {
  matchId: string;
  playerX?: string;
  playerO?: string;
  boardgameState?: string;
  uniqueKey?: string;
}

export interface IMakeMoveDto {
  matchId: string;
  position: number;
  boardgameState?: string;
  uniqueKey?: string;
}

export interface IFetchMatchesDto {
  player?: string;
  matchId?: string;
  bookmark?: string;
  limit?: number;
  uniqueKey?: string;
}

export interface IFetchMatchesResDto {
  results: TicTacMatch[];
  bookmark: string;
}
