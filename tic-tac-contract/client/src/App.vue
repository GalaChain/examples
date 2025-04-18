<template>
  <div class="container">
    <h1>Tic Tac Toe</h1>

    <div v-if="!matchID" class="start-section">
      <button @click="startNewMatch" class="start-button">Start New Game</button>
      <div class="join-section">
        <input v-model="joinMatchId" placeholder="Enter Game ID" class="game-id-input" />
        <button @click="joinMatch" class="join-button" :disabled="!joinMatchId">Join Game</button>
      </div>
    </div>

    <div v-else class="game-section">
      <div class="game-info">
        <p>Game ID: {{ matchID }}</p>
        <p>Current Player: {{ currentPlayer === '0' ? 'X' : 'O' }}</p>
      <p>You are: {{ currentSymbol }}</p>
      </div>

      <div class="board">
        <div v-for="(cell, index) in board"
             :key="index"
             class="cell"
             :class="{ 'cell-playable': isPlayable(index) }"
             @click="makeMove(index)">
          {{ cell === '0' ? 'X' : cell === '1' ? 'O' : '' }}
        </div>
      </div>

      <div v-if="winner || isDraw" class="game-over">
        <div :class="['winner-message', winner !== null ? (isWinner ? 'winner-won' : 'winner-lost') : 'winner-draw']">
          {{ winner !== null ? (isWinner ? 'You won!' : 'You lost!') : 'Game ended in a draw!' }}
        </div>
        <button @click="resetGame" class="play-again-button">Reset</button>
      </div>
    </div>
    <div v-if="!metamaskSupport">
      <p>
        This application uses the GalaConnect API via Metamask to sign
        transactions and interact with GalaChain.
      </p>
      <p>
        Visit this site using a browser with the Metamask web extension
        installed to save game state on chain.
      </p>
    </div>
    <div v-else-if="!isConnected" class="connect-section">
      <button @click="connect">Connect Wallet</button>
    </div>
    <div v-else>
      <p class="wallet-address">Connected: {{ walletAddress }}</p>
      <RouterView
        :wallet-address="walletAddress"
        :metamask-client="metamaskClient"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { Client } from 'boardgame.io/client';
import { SocketIO } from 'boardgame.io/multiplayer';
import { TicTacContract, TicTacContractState } from './game';
import { serialize } from "@gala-chain/api";
import { BrowserConnectClient } from "@gala-chain/connect";
import { connectWallet } from "./connect";
import type { ICreateMatchDto, IJoinMatchDto, IMakeMoveDto } from "./dtos";

const metamaskSupport = ref(true);
let metamaskClient: BrowserConnectClient;
try {
  metamaskClient = new BrowserConnectClient();
} catch (e) {
  metamaskSupport.value = false;
}

const isConnected = ref(false);
const walletAddress = ref("");
const showInfo = ref(false);

async function connect() {
  await connectWallet(metamaskSupport, metamaskClient, walletAddress, isConnected);
}

interface GameOver {
  winner: string | null;
  draw?: boolean;
}

interface ClientState {
  G: TicTacContractState;
  ctx: {
    currentPlayer: string;
    gameover?: GameOver;
  };
  isActive?: boolean;
}

type TicTacContractClient = ReturnType<typeof Client<TicTacContractState, Record<string, unknown>>>;

const serverBaseUrl = import.meta.env.VITE_PROJECT_API ?? 'http://localhost:8000';
const projectId = import.meta.env.VITE_PROJECT_ID ?? 'tic-tac-contract';

const matchID = ref('');
const joinMatchId = ref('');
const loading = ref(false);
const currentPlayer = ref<string>('0');
const board = ref<(string | null)[]>(Array(9).fill(null));
const winner = ref<string | null>(null);
const isDraw = ref(false);
const playerID = ref<string>('0');
const playerName = ref<string>('');
const playerCredentials = ref<string>('');
const client = ref<TicTacContractClient | null>(null);

const isWinner = computed(() => winner.value === playerID.value);
const currentSymbol = computed(() => playerID.value === '0' ? 'X' : 'O');

let unsubscribe: Function | undefined;
let boardgameState: string | undefined;

const initializeClient = (matchID: string, initialPlayerId: string, credentials: string) => {
  playerID.value = initialPlayerId;
  console.log('Initializing client with:', { matchID, initialPlayerId });

  client.value = Client<TicTacContractState, Record<string, unknown>>({
    game: TicTacContract,
    matchID: matchID,
    playerID: playerID.value,
    credentials: credentials,
    debug: false,
    multiplayer: SocketIO({ server: serverBaseUrl })
  });

  console.log('Client initialized:', {
    playerID: client.value.playerID,
    matchID: client.value.matchID
  });

  unsubscribe = client.value.subscribe((state: ClientState | null) => {
    if (state) {
      console.log('Game State Update:', {
        board: state.G.board,
        currentPlayer: state.ctx.currentPlayer,
        winner: state.ctx.gameover?.winner ?? null,
        draw: state.ctx.gameover?.draw,
        clientPlayerID: client.value?.playerID
      });

      board.value = state.G.board;
      currentPlayer.value = state.ctx.currentPlayer;
      winner.value = state.ctx.gameover?.winner ?? null;
      isDraw.value = (state.ctx.gameover?.draw) ?? false;

      try {
        boardgameState = JSON.stringify(state);
      } catch (e) {
        console.log(
          `Received unexpected boardgame.io state that failed to stringify to JSON: ${e}`
        );
      }

      if (state.ctx.gameover) {
        console.log('Game Over State:', {
          winner: state.ctx.gameover.winner,
          currentPlayerId: client.value?.playerID,
          isWinner: state.ctx.gameover.winner === client.value?.playerID
        });
      }
    } else {
      boardgameState = undefined;
    }
  });

  client.value.start();
};

const startNewMatch = async () => {
  loading.value = true;
  try {
    let dto;

    try {
      const signedDto = await confirmCreateMatch();
      dto = signedDto; // JSON.stringify(signedDto);
    } catch (e) {
      // todo: error messaging
      console.log(`Failed to confirm dto signging or stringify signed to: ${e}`);
      loading.value = false;
      return;
    }

    const createGameUrl = `${serverBaseUrl}/games/tic-tac-contract/create`;
    console.log(`POST to /create endpont: ${createGameUrl}`);

    const matchCreatorID = '0';

    const response = await fetch(createGameUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ matchID: dto.matchID, numPlayers: 2, setupData: { dto: dto } })
    });

    const data = await response.json();
    console.log(`create response: ${JSON.stringify(data)}`);

    matchID.value = data.matchID;

    const joinResponse = await fetch(`${serverBaseUrl}/games/tic-tac-contract/${data.matchID}/join`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        playerName: walletAddress.value,
        playerID: matchCreatorID,
        data: {
          authorization: serialize(dto)
        }
      })
    });

    const joinData = await joinResponse.json();
    console.log(`join response: ${JSON.stringify(joinData)}`);

    playerCredentials.value = joinData.playerCredentials;

    initializeClient(matchID.value, matchCreatorID, playerCredentials.value);
    loading.value = false;
  } catch (error) {
    console.error('Failed to start new game:', error);
    loading.value = false;
  }
};

const isPlayable = (index: number): boolean => {
  return !board.value[index] &&
         !winner.value &&
         !isDraw.value &&
         playerID.value === currentPlayer.value;
};

const makeMove = async (index: number) => {
  if (!client.value || !isPlayable(index)) return;

  client.value.moves.makeMove(index);
};

const resetGame = () => {
  matchID.value = '';
  currentPlayer.value = '0';
  board.value = Array(9).fill(null);
  winner.value = null;
  isDraw.value = false;
  playerID.value = '0';
  if (client.value) {
    client.value.stop();
    client.value = null;
  }
  if (unsubscribe !== undefined) {
    unsubscribe();
    unsubscribe = undefined;
  }
};

const joinMatch = async () => {
  if (!joinMatchId.value) return;
  matchID.value = joinMatchId.value;

  const joiningPlayerID = '1';

  const dto = await confirmJoinMatch();

  const joinResponse = await fetch(`${serverBaseUrl}/games/tic-tac-contract/${matchID.value}/join`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      playerName: walletAddress.value,
      playerID: joiningPlayerID,
      data: {
        authorization: serialize(dto)
      }
    })
  });

  const joinData = await joinResponse.json();
  console.log(`join response: ${JSON.stringify(joinData)}`);

  playerCredentials.value = joinData.playerCredentials;

  initializeClient(joinMatchId.value, joiningPlayerID, playerCredentials.value);
  joinMatchId.value = '';
};

async function confirmCreateMatch() {
  // todo: for now, assume match creator is always "X"
  // this could be extended to allow for choice between "X" and "O"

  const matchID: string = `${projectId}-${Date.now()}-${Math.floor(Math.random() * 1000)}`;

  const dto: ICreateMatchDto = {
    matchID: matchID,
    playerX: walletAddress.value,
    uniqueKey: matchID
  }

  if (typeof boardgameState === "string") {
    dto.boardgameState = boardgameState;
  }

  const signedDto = await metamaskClient.sign("CreateMatch", dto);

  console.log("Signed by Gala Connect: ", JSON.stringify(signedDto));

  return signedDto;
}

async function confirmJoinMatch() {
  // todo: for now, assume match creator is always "X"
  // and joiner is "O"
  // this could be extended to allow for choice between "X" and "O"
  const dto: IJoinMatchDto = {
    matchID: matchID.value,
    playerO: walletAddress.value,
    uniqueKey: `${projectId}-${Date.now()}-${Math.floor(Math.random() * 1000)}`
  }

  const signedDto = await metamaskClient.sign("JoinMatch", dto);

  console.log("Signed by Gala Connect: ", JSON.stringify(signedDto));

  return signedDto;
}

async function confirmMakeMove(position: number) {
  const dto: IMakeMoveDto = {
    matchID: matchID.value,
    position: position,
    uniqueKey: `${projectId}-${Date.now()}-${Math.floor(Math.random() * 1000)}`
  }

  if (typeof boardgameState === "string") {
    dto.boardgameState = boardgameState;
  }

  const signedDto = await metamaskClient.sign("MakeMove", dto);

  console.log("Signed by Gala Connect: ", JSON.stringify(signedDto));

  return signedDto;
}
</script>

<style scoped>
.container {
  max-width: 600px;
  margin: 0 auto;
  padding: 20px;
  text-align: center;
}

.board {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
  margin: 20px auto;
  max-width: 300px;
}

.cell {
  aspect-ratio: 1;
  background: #fff;
  border: 2px solid #333;
  font-size: 2em;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background-color 0.2s;
}

.cell-playable {
  cursor: pointer;
}

.cell-playable:hover {
  background-color: #f0f0f0;
}

.cell:not(.cell-playable) {
  cursor: not-allowed;
}

.start-button {
  padding: 10px 20px;
  font-size: 1.2em;
  background-color: #4CAF50;
  color: white;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.start-button:hover {
  background-color: #45a049;
}

.game-info {
  margin: 20px 0;
}

.winner-message {
  margin-top: 20px;
  font-size: 1.5em;
  font-weight: bold;
}

.winner-won {
  color: #4CAF50;
}

.winner-lost {
  color: #f44336;
}

.winner-draw {
  color: #ff9800;
}

.game-over {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  margin-top: 20px;
}

.play-again-button {
  padding: 10px 20px;
  font-size: 1.2em;
  background-color: #9c27b0;
  color: white;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.play-again-button:hover {
  background-color: #7b1fa2;
}

.join-section {
  margin-top: 20px;
  display: flex;
  gap: 10px;
  justify-content: center;
}

.game-id-input {
  padding: 8px;
  border: 2px solid #ddd;
  border-radius: 5px;
  font-size: 1em;
}

.join-button {
  padding: 8px 16px;
  font-size: 1em;
  background-color: #2196F3;
  color: white;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.join-button:hover {
  background-color: #1976D2;
}

.join-button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}
</style>
