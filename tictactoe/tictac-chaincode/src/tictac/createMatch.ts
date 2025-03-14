/*
 * Copyright (c) Gala Games Inc. All rights reserved.
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */
import { GalaChainContext, putChainObject } from "@gala-chain/chaincode";

import { TicTacMatch } from "./TicTacMatch";
import { CreateMatchDto } from "./dtos";

export async function createMatch(ctx: GalaChainContext, dto: CreateMatchDto): Promise<TicTacMatch> {
  const { playerO, playerX, boardgameState } = dto;

  // todo: consider adding validation to verify calling user is either playerO or playerX

  const matchId = dto.matchId;
  const timestamp = ctx.txUnixTime;

  const game = new TicTacMatch(matchId, playerX, playerO, timestamp, boardgameState);
  await putChainObject(ctx, game);

  return game;
}
