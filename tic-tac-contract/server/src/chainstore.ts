import { LogEntry, Server, State, StorageAPI } from 'boardgame.io';
import { Async } from 'boardgame.io/internal';
import { createValidDTO, createValidSubmitDTO, serialize } from "@gala-chain/api";
import { instanceToInstance, plainToInstance } from "class-transformer";
import {
  MatchStateLogOperation,
  MatchStateLogEntry,
  MatchStatePlugin,
  MatchStateContext,
  MatchGameState,
  MatchState,
  MatchStateDto,
  MatchPlayerMetadata,
  MatchMetadata,
  MatchDto,
  FetchMatchDto,
  FetchMatchesDto,
  FetchMatchesResDto,
  FetchMatchIdsResDto,
  CreateMatchDto
} from "./dtos";

import { adminSigningKey } from "./identities";

/**
 * Configuration interface for Chainstore storage adapter
 * 
 * @public
 */
export interface ChainstoreConfig {
  /** Base URL for the GalaChain API endpoint */
  apiUrl?: string;
  /** Contract path for TicTac contract endpoints */
  contractPath?: string;
  /** Custom endpoint names for contract methods */
  endpoints?: {
    /** Endpoint for creating new matches */
    createMatch?: string;
    /** Endpoint for updating match state */
    setMatchState?: string;
    /** Endpoint for updating match metadata */
    setMatchMetadata?: string;
    /** Endpoint for fetching match data */
    fetchMatch?: string;
    /** Endpoint for listing multiple matches */
    fetchMatches?: string;
  };
}

/**
 * Storage Adapter for boardgame.io that uses GalaChain as a backend
 * 
 * This class implements the boardgame.io Async storage interface to persist
 * game state and metadata on the GalaChain blockchain. It handles serialization
 * of game state to DTOs and manages HTTP requests to the chain API.
 * 
 * Influenced by the flatfile.ts example provided in the boardgame.io project.
 * 
 * @example
 * ```typescript
 * const chainstore = new Chainstore({
 *   apiUrl: 'http://localhost:3000',
 *   contractPath: '/api/product/TicTacContract'
 * });
 * 
 * const server = Server({
 *   games: [TicTacContract],
 *   db: chainstore
 * });
 * ```
 * 
 * @public
 */
export class Chainstore extends Async {
  private apiUrl: string;
  private contractPath: string;
  private endpoints: {
    createMatch: string;
    setMatchState: string;
    setMatchMetadata: string;
    fetchMatch: string;
    fetchMatches: string;
  };
  private requestQueues: { [key: string]: Promise<any> };

  /**
   * Creates a new Chainstore instance
   * 
   * @param config - Optional configuration for API endpoints and paths
   */
  constructor(config?: ChainstoreConfig) {
    super();
    this.apiUrl = config?.apiUrl ?? "http://localhost:3000";
    this.contractPath = config?.contractPath ?? "/api/product/TicTacContract";
    this.endpoints = {
      createMatch: config?.endpoints?.createMatch ?? "CreateMatch",
      setMatchState: config?.endpoints?.setMatchState ?? "SetMatchState",
      setMatchMetadata: config?.endpoints?.setMatchMetadata ?? "SetMatchMetadata",
      fetchMatch: config?.endpoints?.fetchMatch ?? "FetchMatch",
      fetchMatches: config?.endpoints?.fetchMatches ?? "FetchMatches"
    };
    this.requestQueues = {};
  }

  /**
   * Chains requests for the same key to ensure sequential execution
   * 
   * This prevents race conditions when multiple operations target the same match ID
   * by ensuring requests are processed in order.
   * 
   * @param key - Unique identifier for the request chain (typically matchID)
   * @param request - Async function to execute
   * @returns Promise that resolves with the request result
   * @private
   */
  private async chainRequest(
    key: string,
    request: () => Promise<any>
  ): Promise<any> {
    if (!(key in this.requestQueues)) this.requestQueues[key] = Promise.resolve();

    // chains the current promise onto the resolution of any previous/pending promise
    this.requestQueues[key] = this.requestQueues[key].then(request, request);

    return this.requestQueues[key];
  }

  /**
   * Establishes connection to the storage backend
   * 
   * For HTTP-based GalaChain API, this is currently a no-op.
   * Consider implementing a health check or version endpoint query in the future.
   * 
   * @returns Promise that resolves when connection is established
   * @public
   */
  async connect(): Promise<void> {
    // No-op for HTTP API
    // todo: consider querying a health check or version endpoint of the target chain/ops deployment
    return;
  }

  /**
   * Creates a new match on the blockchain
   * 
   * Serializes the initial game state and metadata to GalaChain DTOs,
   * then submits a signed transaction to create the match on-chain.
   * 
   * @param matchID - Unique identifier for the match
   * @param opts - Match creation options including initial state and metadata
   * @returns Promise that resolves when match is created
   * @throws Error if match creation fails
   * @public
   */
  async createMatch(
    matchID: string,
    opts: StorageAPI.CreateMatchOpts
  ): Promise<void> {
    const { initialState, metadata } = opts;
    const initialStateID = InitialStateKey(matchID);

    console.log(`createMatch:`);
    console.log(`initialState: ${JSON.stringify(initialState)}`);
    console.log(`metadata: ${JSON.stringify(metadata)}`)

    const { G, ctx, _stateID, plugins } = initialState;

    const stateDto = await createValidDTO(MatchState, {
      _stateID: _stateID,
      G: plainToInstance(MatchGameState, G),
      ctx: plainToInstance(MatchStateContext, ctx),
      plugins: plugins as Record<string, MatchStatePlugin>
    });

    const matchPlayers: Record<number, MatchPlayerMetadata> = {};

    for (const p in metadata.players) {
      matchPlayers[p] = plainToInstance(MatchMetadata, metadata.players[p])
    }

    const metadataDto = await createValidDTO(MatchMetadata, {
      ...metadata,
      players: matchPlayers
    });

    const dto = (await createValidDTO(CreateMatchDto, {
      matchID,
      initialStateID,
      state: stateDto,
      metadata: metadataDto,
      uniqueKey: `${matchID}-${Date.now()}-${Math.floor(Math.random() * 10000)}`
    })).signed(adminSigningKey());

    return await this.chainRequest(matchID, async () => {

      const url = `${this.apiUrl}${this.contractPath}/${this.endpoints.createMatch}`;
      console.log(`createMatch request url: ${url}`);
      console.log(`createMatch with data: ${dto.serialize()}`);

      const response = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: dto.serialize()
      });

      if (!response.ok) {
        const error = await response.text();
        console.log(`createMatch request failed: ${error}`);
        throw new Error(`Failed to create match ${matchID} on chain: ${error}`);
      }

      const data = await response.json();
      console.log(`createMatch success: ${JSON.stringify(data)}`);
    });
  }

  async fetch<O extends StorageAPI.FetchOpts>(
    matchID: string,
    { state, log, metadata, initialState }: O
  ): Promise<StorageAPI.FetchResult<O>> {
    return this.chainRequest(matchID, async () => {
      const dto = new FetchMatchDto();
      dto.matchID = matchID;
      dto.includeLog = log;
      dto.includeState = state;
      dto.includeMetadata = metadata;
      dto.includeInitialState = initialState;

      const url = `${this.apiUrl}${this.contractPath}/${this.endpoints.fetchMatch}`;
      console.log(`fetch: ${url}`);
      console.log(
        `fetch options: state? ${state}; log? ${log}; metadata? ${metadata}; initialState? ${initialState}`
      );
      const response = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(dto)
      });

      if (!response.ok) {
        console.log(`fetch to ${url} failed.`);
        const responseText = await response.text();
        console.log(`response: ${responseText}`);
        return {};
      }

      const chainRes = await response.json();

      console.log(`fetch success: ${JSON.stringify(chainRes)}`);

      return chainRes.Data as StorageAPI.FetchResult<O>;
    });
  }

  async clear() {
    // todo: implement if needed
    return;
  }

  async setState(
    matchID: string,
    state: State,
    deltalog?: LogEntry[]
  ): Promise<void> {
    const { G, ctx, _stateID, plugins, _undo, _redo } = state;
    return await this.chainRequest(matchID, async () => {
      const stateDto = await createValidDTO(MatchState, {
        _stateID: _stateID,
        G: plainToInstance(MatchGameState, G),
        ctx: plainToInstance(MatchStateContext, ctx),
        plugins: plugins as Record<string, MatchStatePlugin>
      });

      const deltalogDto: MatchStateLogEntry[] | undefined = deltalog ?
        deltalog.map((d) => plainToInstance(MatchStateLogEntry, d)) :
        deltalog;

      const dto = (await createValidDTO(MatchStateDto, {
        matchID,
        state: stateDto,
        deltalog: deltalogDto,
        uniqueKey: `${matchID}-${Date.now()}-${Math.floor(Math.random() * 10000)}`
      })).signed(adminSigningKey());

      const url = `${this.apiUrl}${this.contractPath}/${this.endpoints.setMatchState}`;
      console.log(`setState request url: ${url}`);
      console.log(`setState with data: ${dto.serialize()}`);

      const response = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: dto.serialize()
      });

      if (!response.ok) {
        const error = await response.text();
        console.log(`setState request failed: ${error}`);
        throw new Error(`Failed to set state for match ${matchID} on chain: ${error}`);
      }

      const data = await response.json();
      console.log(`setState success: ${JSON.stringify(data)}`);

    });
  }

  async setMetadata(
    matchID: string,
    metadata: Server.MatchData
  ): Promise<void> {
    return await this.chainRequest(matchID, async () => {
      const matchPlayers: Record<number, MatchPlayerMetadata> = {};

      for (const p in metadata.players) {
        matchPlayers[p] = plainToInstance(MatchMetadata, metadata.players[p])
      }

      const metadataDto = await createValidDTO(MatchMetadata, {
        ...metadata,
        players: matchPlayers
      });

      const dto = (await createValidDTO(MatchStateDto, {
        matchID,
        metadata: metadataDto,
        uniqueKey: `${matchID}-${Date.now()}-${Math.floor(Math.random() * 10000)}`
      })).signed(adminSigningKey())

      const url = `${this.apiUrl}${this.contractPath}/${this.endpoints.setMatchMetadata}`;

      console.log(`setMetadata request: ${url}`);
      console.log(dto.serialize());

      const response = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: dto.serialize()
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(`Failed to set state for match ${matchID} on chain: ${error}`);
      }
    });
  }

  async wipe(matchID: string): Promise<void> {
    // todo: implement deleteState methods on chain
    // await this.removeItem(matchID);
    // await this.removeItem(InitialStateKey(matchID))
    // await this.removeItem(LogKey(matchID));
    // await this.removeItem(MetadataKey(matchID));
    return;
  }

  async listMatches(opts?: StorageAPI.ListMatchesOpts): Promise<string[]> {
    const dto = new FetchMatchesDto();

    const url = `${this.apiUrl}${this.contractPath}/${this.endpoints.fetchMatches}`;

    console.log(`listMatches url: ${url}`);

    const response = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(dto)
    });

    if (!response.ok) {
      throw new Error('Failed to list matches from chain');
    }

    const { results } = await response.json();

    console.log(`listMatches response: ${JSON.stringify(results)}`);

    return results;
  }
}

// todo: move these values to contants / enum type
// and rather than following a single string as in flat file, e.g
// `${matchID}:initial` with GalaChain composite keys we use
// separate parts ...
export function InitialStateKey(matchID: string) {
  return `${matchID}:initial`;
}

export function MetadataKey(matchID: string) {
  return `${matchID}:metadata`
}

export function LogKeySuffix(matchID: string) {
  return `${matchID}:log`;
}
