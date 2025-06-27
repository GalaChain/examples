import {
  GalaChainContext,
  GalaContract,
  GalaTransaction,
  GalaTransactionType,
  UnsignedEvaluate
} from "@gala-chain/chaincode";

import { version } from "../../package.json";
import { TicTacMatch } from "./TicTacMatch";
import { createMatch } from "./createMatch";
import {
  CreateMatchDto,
  FetchMatchDto,
  FetchMatchesDto,
  FetchMatchesResDto,
  JoinMatchDto,
  MatchDto
} from "./dtos";
import { fetchMatch } from "./fetchMatch";
import { fetchMatches } from "./fetchMatches";
import { joinMatch } from "./joinMatch";
import { setMatchMetadata } from "./setMatchMetadata";
import { setMatchState } from "./setMatchState";

/**
 * GalaChain contract for Tic Tac Toe game management
 * 
 * This contract handles all blockchain operations for tic-tac-toe games,
 * including match creation, state updates, player management, and data retrieval.
 * It integrates with boardgame.io for game logic while persisting state on-chain.
 * 
 * @example
 * ```typescript
 * // Create a new match
 * const createDto = new CreateMatchDto();
 * createDto.matchID = "game-123";
 * const result = await contract.CreateMatch(ctx, createDto);
 * 
 * // Fetch match data
 * const fetchDto = new FetchMatchDto();
 * fetchDto.matchID = "game-123";
 * const match = await contract.FetchMatch(ctx, fetchDto);
 * ```
 * 
 * @public
 */
export class TicTacContract extends GalaContract {
  /**
   * Creates a new TicTacContract instance
   */
  constructor() {
    super("TicTacContract", version);
  }

  /**
   * Creates a new tic-tac-toe match on the blockchain
   * 
   * Initializes a new game with the provided state and metadata,
   * storing both match metadata and initial game state on-chain.
   * 
   * @param ctx - GalaChain transaction context
   * @param dto - Match creation data including initial state and metadata
   * @returns The created match DTO
   * @throws Error if match creation fails or duplicate match ID
   */
  @GalaTransaction({
    in: CreateMatchDto,
    out: CreateMatchDto,
    type: GalaTransactionType.SUBMIT,
    verifySignature: true,
    enforceUniqueKey: true
  })
  public async CreateMatch(ctx: GalaChainContext, dto: CreateMatchDto): Promise<CreateMatchDto> {
    return createMatch(ctx, dto);
  }

  /**
   * Retrieves match data from the blockchain
   * 
   * Fetches complete match information including current state,
   * metadata, and player information for the specified match ID.
   * 
   * @param ctx - GalaChain evaluation context
   * @param dto - Fetch request containing match ID
   * @returns Complete match data including state and metadata
   * @throws Error if match not found
   */
  @UnsignedEvaluate({
    in: FetchMatchDto,
    out: MatchDto
  })
  public async FetchMatch(ctx: GalaChainContext, dto: FetchMatchDto): Promise<MatchDto> {
    return fetchMatch(ctx, dto);
  }

  @GalaTransaction({
    in: JoinMatchDto,
    out: TicTacMatch,
    type: GalaTransactionType.SUBMIT,
    verifySignature: true,
    enforceUniqueKey: true
  })
  public async JoinMatch(ctx: GalaChainContext, dto: JoinMatchDto): Promise<TicTacMatch> {
    return joinMatch(ctx, dto);
  }

  @GalaTransaction({
    in: MatchDto,
    out: TicTacMatch,
    type: GalaTransactionType.SUBMIT,
    verifySignature: true,
    enforceUniqueKey: true
  })
  public async SetMatchState(ctx: GalaChainContext, dto: MatchDto): Promise<TicTacMatch> {
    return setMatchState(ctx, dto);
  }

  @GalaTransaction({
    in: MatchDto,
    out: MatchDto,
    type: GalaTransactionType.SUBMIT,
    verifySignature: true,
    enforceUniqueKey: true
  })
  public async SetMatchMetadata(ctx: GalaChainContext, dto: MatchDto): Promise<MatchDto> {
    return setMatchMetadata(ctx, dto);
  }

  @UnsignedEvaluate({
    in: FetchMatchesDto,
    out: FetchMatchesResDto
  })
  public async FetchMatches(ctx: GalaChainContext, dto: FetchMatchesDto): Promise<FetchMatchesResDto> {
    return await fetchMatches(ctx, dto);
  }
}
