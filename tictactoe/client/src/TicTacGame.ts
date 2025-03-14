/*
 * Copyright (c) Gala Games Inc. All rights reserved.
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */
import { ChainKey, ChainObject, DefaultError, NotFoundError } from "@gala-chain/api";
import { Exclude } from "class-transformer";
import { IsArray, IsString } from "class-validator";

import { GameStatus, PlayerSymbol } from "./types";

export class TicTacMatch extends ChainObject {
  @Exclude()
  static INDEX_KEY = "GCTTT";

  @ChainKey({ position: 0 })
  @IsString()
  public readonly matchId: string;

  @IsString()
  public readonly playerX: string;

  @IsString()
  public readonly playerO: string;

  @IsArray()
  public board: (PlayerSymbol | null)[];

  public status: GameStatus;

  public currentPlayer: PlayerSymbol;

  public readonly createdAt: number;

  public lastMoveAt: number;

  constructor(matchId: string, playerX: string, playerO: string, createdAt: number) {
    super();
    this.matchId = matchId;
    this.playerX = playerX;
    this.playerO = playerO;
    this.board = Array(9).fill(null);
    this.status = GameStatus.IN_PROGRESS;
    this.currentPlayer = PlayerSymbol.X;
    this.createdAt = createdAt;
    this.lastMoveAt = createdAt;
  }

  private checkWinner(): GameStatus {
    const lines = [
      [0, 1, 2],
      [3, 4, 5],
      [6, 7, 8], // rows
      [0, 3, 6],
      [1, 4, 7],
      [2, 5, 8], // columns
      [0, 4, 8],
      [2, 4, 6] // diagonals
    ];

    for (const [a, b, c] of lines) {
      if (this.board[a] && this.board[a] === this.board[b] && this.board[a] === this.board[c]) {
        return this.board[a] === PlayerSymbol.X ? GameStatus.X_WON : GameStatus.O_WON;
      }
    }

    if (this.board.every((cell) => cell !== null)) {
      return GameStatus.DRAW;
    }

    return GameStatus.IN_PROGRESS;
  }

  public canMakeMove(player: string, position: number): boolean {
    return (
      this.status === GameStatus.IN_PROGRESS &&
      position >= 0 &&
      position < 9 &&
      this.board[position] === null &&
      ((this.currentPlayer === PlayerSymbol.X && player === this.playerX) ||
        (this.currentPlayer === PlayerSymbol.O && player === this.playerO))
    );
  }

  public makeMove(player: string, position: number, timestamp: number): void {
    if (!this.canMakeMove(player, position)) {
      throw new InvalidMoveError(player, position, this.currentPlayer, this.status);
    }

    this.board[position] = this.currentPlayer;
    this.lastMoveAt = timestamp;

    // Update game status
    const newStatus = this.checkWinner();
    if (newStatus !== GameStatus.IN_PROGRESS) {
      this.status = newStatus;
    } else {
      // Switch current player
      this.currentPlayer = this.currentPlayer === PlayerSymbol.X ? PlayerSymbol.O : PlayerSymbol.X;
    }
  }
}

class InvalidMoveError extends DefaultError {
  constructor(player: string, position: number, currentPlayer: PlayerSymbol, status: GameStatus) {
    super("Invalid move", { player, position, currentPlayer, status });
  }
}
