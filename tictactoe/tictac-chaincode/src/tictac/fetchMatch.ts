/*
 * Copyright (c) Gala Games Inc. All rights reserved.
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */
import { GalaChainContext, getObjectByKey } from "@gala-chain/chaincode";

import { TicTacMatch } from "./TicTacMatch";
import { FetchMatchDto } from "./dtos";

export async function fetchMatch(ctx: GalaChainContext, dto: FetchMatchDto): Promise<TicTacMatch> {
  const gameKey = TicTacMatch.getCompositeKeyFromParts(TicTacMatch.INDEX_KEY, [dto.matchId]);
  const match = await getObjectByKey(ctx, TicTacMatch, gameKey);

  return match;
}
