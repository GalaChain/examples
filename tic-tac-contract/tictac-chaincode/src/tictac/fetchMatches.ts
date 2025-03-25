/*
 * Copyright (c) Gala Games Inc. All rights reserved.
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */
import { createValidDTO } from "@gala-chain/api";
import {
  GalaChainContext,
  getObjectsByPartialCompositeKeyWithPagination,
  takeUntilUndefined
} from "@gala-chain/chaincode";

import { TicTacMatch } from "./TicTacMatch";
import { FetchMatchesDto, FetchMatchesResDto } from "./dtos";

export async function fetchMatches(ctx: GalaChainContext, dto: FetchMatchesDto): Promise<FetchMatchesResDto> {
  const query = takeUntilUndefined(dto.matchID);

  const lookup = await getObjectsByPartialCompositeKeyWithPagination(
    ctx,
    TicTacMatch.INDEX_KEY,
    query,
    TicTacMatch,
    dto.bookmark ?? "",
    dto.limit ?? 1000
  );

  return await createValidDTO(FetchMatchesResDto, {
    results: lookup.results,
    bookmark: lookup.metadata.bookmark
  });
}
