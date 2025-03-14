/*
 * Copyright (c) Gala Games Inc. All rights reserved.
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */
import { ChainCallDTO, SubmitCallDTO } from "@gala-chain/api";
import { Type } from "class-transformer";
import {
  IsArray,
  IsNotEmpty,
  IsNumber,
  IsOptional,
  IsString,
  Max,
  Min,
  ValidateIf,
  ValidateNested
} from "class-validator";

import { TicTacMatch } from "./TicTacMatch";

export class CreateMatchDto extends SubmitCallDTO {
  @IsNotEmpty()
  public matchId: string;

  @IsOptional()
  @IsString()
  @ValidateIf((o) => o.playerO === undefined)
  public playerX?: string;

  @IsOptional()
  @IsString()
  @ValidateIf((o) => o.playerX === undefined)
  public playerO?: string;

  @IsOptional()
  @IsString()
  public boardgameState?: string;

  constructor(
    playerX: string | undefined,
    playerO: string | undefined,
    uniqueKey: string,
    boardgameState?: string
  ) {
    super();
    this.playerX = playerX;
    this.playerO = playerO;
    this.uniqueKey = uniqueKey;
    this.boardgameState = boardgameState;
  }
}

export class JoinMatchDto extends SubmitCallDTO {
  @IsString()
  public matchId: string;

  @IsOptional()
  @IsString()
  @ValidateIf((o) => o.playerO === undefined)
  public playerX?: string;

  @IsOptional()
  @IsString()
  @ValidateIf((o) => o.playerX === undefined)
  public playerO?: string;

  @IsOptional()
  @IsString()
  public boardgameState?: string;

  constructor(
    matchId: string,
    playerX: string | undefined,
    playerO: string | undefined,
    uniqueKey: string,
    boardgameState?: string
  ) {
    super();
    this.matchId = matchId;
    this.playerX = playerX;
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

export class FetchMatchDto extends ChainCallDTO {
  @IsString()
  public matchId: string;
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

export class FetchMatchesResDto extends ChainCallDTO {
  @IsArray()
  @ValidateNested({ each: true })
  @Type(() => TicTacMatch)
  public readonly results: TicTacMatch[];

  @IsString()
  public readonly bookmark: string;

  constructor(results: TicTacMatch[], bookmark: string) {
    super();
    this.results = results;
    this.bookmark = bookmark;
  }
}
