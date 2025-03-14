# Tic Tac Toe Game

A multiplayer Tic Tac Toe game built with Koa.js, TypeScript, Vue.js, and boardgame.io.

## Project Structure

```
tictactoe/
├── src/
│   ├── client/         # Vue.js client application
│   └── server/         # Koa.js + boardgame.io server
├── dist/               # Compiled output
├── package.json        # Project dependencies
├── tsconfig.json      # TypeScript configuration
└── vite.config.ts     # Vite configuration
```

## Setup

1. Install dependencies:
```bash
npm install
```

2. Start the development server:
```bash
# Start the game server and API
npm run dev:server

# In another terminal, start the client development server
npm run dev:client
```

3. Open your browser and navigate to `http://localhost:5173`

## Building for Production

```bash
# Build the client
npm run build:client

# Build the server
npm run build

# Start the production server
npm start
```
