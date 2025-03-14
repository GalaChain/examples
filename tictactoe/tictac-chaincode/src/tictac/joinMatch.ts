/*
 * Copyright (c) Gala Games Inc. All rights reserved.
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */
import { ConflictError } from "@gala-chain/api";
import { GalaChainContext, getObjectByKey, putChainObject } from "@gala-chain/chaincode";

import { TicTacMatch } from "./TicTacMatch";
import { JoinMatchDto } from "./dtos";

export async function joinMatch(ctx: GalaChainContext, dto: JoinMatchDto): Promise<TicTacMatch> {
  const { matchId, playerO, playerX, boardgameState } = dto;

  // todo: consider adding validation to verify calling user is either playerO or playerX

  const key = TicTacMatch.getCompositeKeyFromParts(TicTacMatch.INDEX_KEY, [matchId]);
  const match = await getObjectByKey(ctx, TicTacMatch, key);

  if (playerO !== undefined && match.playerO === undefined) {
    match.playerO = playerO;
  } else if (playerX !== undefined && match.playerX === undefined) {
    match.playerX = playerX;
  } else {
    throw new ConflictError(
      `Match ${matchId} found, but conflict exists between provided and existing X and O players: ` +
        `Provided(playerX ${playerX}, playerO ${playerO}), ` +
        `Exists(playerX ${match.playerX}, playerO ${match.playerO}`
    );
  }

  if (typeof boardgameState === "string") {
    match.boardgameState = boardgameState;
  }

  await putChainObject(ctx, match);

  return match;
}
