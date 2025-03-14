/*
 * Copyright (c) Gala Games Inc. All rights reserved.
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */
import { GalaChainContext, GalaContract, Submit, UnsignedEvaluate } from "@gala-chain/chaincode";

import { version } from "../../package.json";
import { TicTacMatch } from "./TicTacMatch";
import { createMatch } from "./createMatch";
import {
  CreateMatchDto,
  FetchMatchDto,
  FetchMatchesDto,
  FetchMatchesResDto,
  JoinMatchDto,
  MakeMoveDto
} from "./dtos";
import { fetchMatch } from "./fetchMatch";
import { fetchMatches } from "./fetchMatches";
import { joinMatch } from "./joinMatch";
import { makeMove } from "./makeMove";

export class TicTacContract extends GalaContract {
  constructor() {
    super("TicTacContract", version);
  }

  @Submit({
    in: CreateMatchDto,
    out: TicTacMatch
  })
  public async CreateMatch(ctx: GalaChainContext, dto: CreateMatchDto): Promise<TicTacMatch> {
    return createMatch(ctx, dto);
  }

  @UnsignedEvaluate({
    in: FetchMatchDto,
    out: TicTacMatch
  })
  public async FetchMatch(ctx: GalaChainContext, dto: FetchMatchDto): Promise<TicTacMatch> {
    return fetchMatch(ctx, dto);
  }

  @Submit({
    in: JoinMatchDto,
    out: TicTacMatch
  })
  public async JoinMatch(ctx: GalaChainContext, dto: JoinMatchDto): Promise<TicTacMatch> {
    return joinMatch(ctx, dto);
  }

  @Submit({
    in: MakeMoveDto,
    out: TicTacMatch
  })
  public async MakeMove(ctx: GalaChainContext, dto: MakeMoveDto): Promise<TicTacMatch> {
    return makeMove(ctx, dto);
  }

  @UnsignedEvaluate({
    in: FetchMatchesDto,
    out: FetchMatchesResDto
  })
  public async FetchMatches(ctx: GalaChainContext, dto: FetchMatchesDto): Promise<FetchMatchesResDto> {
    return await fetchMatches(ctx, dto);
  }
}
