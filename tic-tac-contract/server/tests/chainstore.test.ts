/**
 * @fileoverview Tests for the Chainstore class - the critical storage adapter
 * that bridges boardgame.io with GalaChain persistence.
 */

import { ChainUser } from "@gala-chain/api";

import { Chainstore, ChainstoreConfig } from '../src/chainstore';
import { StorageAPI, State, Server } from 'boardgame.io';

// Mock fetch globally  
global.fetch = jest.fn();
const mockFetch = global.fetch as jest.MockedFunction<typeof fetch>;

// Mock the adminSigningKey
const adminUser = ChainUser.withRandomKeys();
jest.mock('../src/identities', () => ({
  adminSigningKey: jest.fn(() => adminUser.privateKey)
}));

describe('Chainstore', () => {
  let chainstore: Chainstore;
  const mockConfig: ChainstoreConfig = {
    apiUrl: 'http://test-api.com',
    contractPath: '/api/test/Contract',
    endpoints: {
      createMatch: 'TestCreateMatch',
      setMatchState: 'TestSetMatchState',
      setMatchMetadata: 'TestSetMatchMetadata',
      fetchMatch: 'TestFetchMatch',
      fetchMatches: 'TestFetchMatches'
    }
  };

  beforeEach(() => {
    chainstore = new Chainstore(mockConfig);
    mockFetch.mockClear();
  });

  describe('constructor', () => {
    it('should initialize with default configuration when no config provided', () => {
      const defaultChainstore = new Chainstore();
      expect(defaultChainstore).toBeInstanceOf(Chainstore);
    });

    it('should initialize with custom configuration', () => {
      expect(chainstore).toBeInstanceOf(Chainstore);
    });
  });

  describe('connect', () => {
    it('should connect successfully (no-op for HTTP API)', async () => {
      await expect(chainstore.connect()).resolves.toBeUndefined();
    });
  });

  describe('createMatch', () => {
    const mockMatchID = 'test-match-123';
    const mockOpts: StorageAPI.CreateMatchOpts = {
      initialState: {
        G: { board: Array(9).fill(null), currentPlayer: 'X' },
        ctx: {
          numPlayers: 2,
          turn: 1,
          currentPlayer: '0',
          phase: 'play',
          playOrder: ['0', '1'],
          playOrderPos: 0,
          activePlayers: null
        },
        _stateID: 0,
        plugins: {},
        _undo: [],
        _redo: []
      },
      metadata: {
        gameName: 'tic-tac-toe',
        players: {
          '0': { id: 0, name: 'Player 1' },
          '1': { id: 1, name: 'Player 2' }
        },
        setupData: {},
        gameover: undefined,
        nextMatchID: undefined,
        unlisted: false,
        createdAt: Date.now(),
        updatedAt: Date.now()
      }
    };

    it('should create match successfully with valid response', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, matchID: mockMatchID })
      } as any);

      await expect(chainstore.createMatch(mockMatchID, mockOpts)).resolves.toBeUndefined();

      expect(mockFetch).toHaveBeenCalledWith(
        'http://test-api.com/api/test/Contract/TestCreateMatch',
        expect.objectContaining({
          method: 'POST',
          headers: { 'Content-Type': 'application/json' }
        })
      );
    });

    it('should throw error when API request fails', async () => {
      const errorMessage = 'Chain API error';
      mockFetch.mockResolvedValueOnce({
        ok: false,
        text: async () => errorMessage
      } as any);

      await expect(chainstore.createMatch(mockMatchID, mockOpts))
        .rejects.toThrow(`Failed to create match ${mockMatchID} on chain: ${errorMessage}`);
    });

    it('should handle network errors', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      await expect(chainstore.createMatch(mockMatchID, mockOpts))
        .rejects.toThrow('Network error');
    });
  });

  describe('setState', () => {
    const mockMatchID = 'test-match-123';
    const mockStateID = 1;
    const mockState: State = {
      G: { board: ['X', null, null, null, null, null, null, null, null], currentPlayer: 'O' },
      ctx: {
        numPlayers: 2,
        turn: 2,
        currentPlayer: '1',
        phase: 'play',
        playOrder: ['0', '1'],
        playOrderPos: 1,
        activePlayers: null
      },
      _stateID: mockStateID,
      plugins: {},
      _undo: [],
      _redo: []
    };

    it('should set state successfully', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true })
      } as any);

      await expect(chainstore.setState(mockMatchID, mockState)).resolves.toBeUndefined();

      expect(mockFetch).toHaveBeenCalledWith(
        'http://test-api.com/api/test/Contract/TestSetMatchState',
        expect.objectContaining({
          method: 'POST',
          headers: { 'Content-Type': 'application/json' }
        })
      );
    });

    it('should throw error when setState fails', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        text: async () => 'State update failed'
      } as any);

      await expect(chainstore.setState(mockMatchID, mockState))
        .rejects.toThrow('Failed to set state for match test-match-123 on chain: State update failed');
    });
  });

  describe('fetch', () => {
    const mockMatchID = 'test-match-123';
    const mockStateID = 1;

    it('should fetch state successfully', async () => {
      const mockResponse = {
        Data: {
          state: {
            G: { board: Array(9).fill(null), currentPlayer: 'X' },
            ctx: { numPlayers: 2, turn: 1, currentPlayer: '0', phase: 'play' },
            _stateID: mockStateID,
            plugins: {}
          },
          metadata: {
            gameName: 'tic-tac-toe',
            players: { '0': { id: '0', name: 'Player 1' } }
          }
        }
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse
      } as any);

      const result = await chainstore.fetch(mockMatchID, { state: true });

      expect(result).toEqual({
        state: expect.objectContaining({
          G: expect.any(Object),
          ctx: expect.any(Object),
          _stateID: mockStateID
        }),
        metadata: expect.objectContaining({
          gameName: 'tic-tac-toe',
          players: expect.any(Object)
        })
      });
    });

    it('should return empty object when match not found', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        text: async () => 'Match not found'
      } as any);

      const result = await chainstore.fetch(mockMatchID, { state: true });
      expect(result).toEqual({});
    });

    it('should return empty object for other API failures', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        text: async () => 'Internal server error'
      } as any);

      const result = await chainstore.fetch(mockMatchID, { state: true });
      expect(result).toEqual({});
    });
  });

  describe('listMatches', () => {
    it('should list matches successfully', async () => {
      const mockMatches = [
        'match-1',
        'match-2'
      ];

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ results: mockMatches })
      } as any);

      const result = await chainstore.listMatches();
      expect(result).toEqual(mockMatches);
    });

    it('should return empty array when no matches found', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ results: [] })
      } as any);

      const result = await chainstore.listMatches();
      expect(result).toEqual([]);
    });
  });

  describe('request chaining', () => {
    it('should serialize requests for the same match ID', async () => {
      const matchID = 'test-match';
      let call1Started = false;
      let call1Finished = false;
      let call2Started = false;

      // Mock delayed responses
      mockFetch
        .mockImplementationOnce(async () => {
          call1Started = true;
          await new Promise(resolve => setTimeout(resolve, 100));
          call1Finished = true;
          return { ok: true, json: async () => ({}) } as any;
        })
        .mockImplementationOnce(async () => {
          call2Started = true;
          expect(call1Finished).toBe(true); // Second call should wait for first
          return { ok: true, json: async () => ({}) } as any;
        });

      const mockOpts: StorageAPI.CreateMatchOpts = {
        initialState: {
          G: {},
          ctx: { 
            numPlayers: 2, 
            turn: 1, 
            currentPlayer: '0', 
            phase: 'play',
            playOrder: ['0', '1'],
            playOrderPos: 0,
            activePlayers: null
          },
          _stateID: 0,
          plugins: {},
          _undo: [],
          _redo: []
        },
        metadata: {
          gameName: 'test',
          players: {},
          setupData: {},
          gameover: undefined,
          nextMatchID: undefined,
          unlisted: false,
          createdAt: Date.now(),
          updatedAt: Date.now()
        }
      };

      // Start both requests simultaneously
      const promise1 = chainstore.createMatch(matchID, mockOpts);
      const promise2 = chainstore.createMatch(matchID, mockOpts);

      await Promise.all([promise1, promise2]);

      expect(call1Started).toBe(true);
      expect(call1Finished).toBe(true);
      expect(call2Started).toBe(true);
    });
  });
});