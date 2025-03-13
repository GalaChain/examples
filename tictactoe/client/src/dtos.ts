/*
 * Copyright (c) Gala Games Inc. All rights reserved.
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */
import { ChainCallDTO, SubmitCallDTO } from "@gala-chain/api";
import { Type } from "class-transformer";
import { IsArray, IsNumber, IsOptional, IsString, Max, Min, ValidateNested } from "class-validator";

import { TicTacMatch } from "./TicTacMatch";

export class CreateMatchDto extends SubmitCallDTO {
  @IsString()
  public playerO: string;

  @IsString()
  @IsOptional()
  public boardgameState?: string;

  constructor(playerO: string, uniqueKey: string, boardgameState?: string) {
    super();
    this.playerO = playerO;
    this.uniqueKey = uniqueKey;
    this.boardgameState = boardgameState;
  }
}

export class MakeMoveDto extends SubmitCallDTO {
  @IsString()
  public matchId: string;

  @IsNumber()
  @Min(0)
  @Max(8)
  public position: number;

  @IsString()
  @IsOptional()
  public boardgameState?: string;

  constructor(matchId: string, position: number, uniqueKey: string, boardgameState?: string) {
    super();
    this.matchId = matchId;
    this.position = position;
    this.uniqueKey = uniqueKey;
    this.boardgameState = boardgameState;
  }
}

export class FetchMatchesDto extends ChainCallDTO {
  @IsString()
  @IsOptional()
  public player?: string;

  @IsString()
  @IsOptional()
  public matchId?: string;

  @IsString()
  @IsOptional()
  public bookmark?: string;

  @IsOptional()
  public limit?: number;

  constructor(player?: string, matchId?: string, bookmark?: string, limit?: number) {
    super();
    this.player = player;
    this.matchId = matchId;
    this.bookmark = bookmark;
    this.limit = limit;
  }
}

export class PagedGamesDto {
  @IsArray()
  @ValidateNested({ each: true })
  @Type(() => TicTacMatch)
  public games: TicTacMatch[];

  @IsString()
  public bookmark: string;

  constructor(games: TicTacMatch[], bookmark: string) {
    this.games = games;
    this.bookmark = bookmark;
  }
}
