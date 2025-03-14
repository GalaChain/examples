/*
 * Copyright (c) Gala Games Inc. All rights reserved.
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */
import { GalaChainContext, getObjectByKey, putChainObject } from "@gala-chain/chaincode";

import { TicTacMatch } from "./TicTacMatch";
import { MakeMoveDto } from "./dtos";

export async function makeMove(ctx: GalaChainContext, dto: MakeMoveDto): Promise<TicTacMatch> {
  const player = ctx.callingUser;
  const timestamp = ctx.txUnixTime;
  const boardgameState = dto.boardgameState;
  // Get the game
  const gameKey = TicTacMatch.getCompositeKeyFromParts(TicTacMatch.INDEX_KEY, [dto.matchId]);
  const match = await getObjectByKey(ctx, TicTacMatch, gameKey);

  // Make the move
  match.makeMove(player, dto.position, timestamp);

  if (typeof boardgameState === "string") {
    match.boardgameState = boardgameState;
  }
  // Save the updated game state
  await putChainObject(ctx, match);

  return match;
}
