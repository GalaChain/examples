# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Architecture

This is a sophisticated Tic Tac Toe implementation demonstrating boardgame.io integration with GalaChain blockchain state persistence. The project consists of three interconnected applications:

### Core Components

1. **Client** (`/client/`): Vue.js frontend with TypeScript
   - Uses Vite for development and building
   - Integrates with boardgame.io client library
   - Connects to GalaChain via `@gala-chain/connect`

2. **Server** (`/server/`): Koa.js backend with custom storage adapter
   - Implements boardgame.io server with custom `Chainstore` adapter
   - Bridges boardgame.io state management with GalaChain persistence
   - Key file: `src/chainstore.ts` - custom storage adapter implementing boardgame.io's Async interface

3. **Chaincode** (`/tictac-chaincode/`): GalaChain smart contract
   - Implements game state persistence on blockchain
   - Uses GalaChain framework with TypeScript
   - Key contract: `src/tictac/TicTacContract.ts`

### Key Integration Pattern

The project uses a unique architecture where:
- boardgame.io manages game logic and turn mechanics
- Custom `Chainstore` adapter serializes game state to GalaChain
- Business logic is replayed and re-verified on-chain for integrity
- DTOs ensure type safety across client/server/chain boundaries

## Development Commands

### Chaincode Development
```bash
cd tictac-chaincode
npm install          # Install dependencies
npm run build        # Build TypeScript to lib/
npm run lint         # ESLint checking
npm run fix          # Auto-fix ESLint issues
npm run format       # Prettier formatting
npm test             # Run Jest unit tests
npm run test:e2e     # Run end-to-end tests
npm run network:up   # Start local GalaChain network
npm run network:start # Start network with watch mode
npm run docs         # Generate TypeDoc documentation
npm run docs:serve   # Generate docs with watch mode
```

### Server Development
```bash
cd server
npm install          # Install dependencies
npm run build        # Build TypeScript
npm run dev          # Development server with hot reload
npm start            # Start production server
npm test             # Run Jest unit tests
npm run test:watch   # Run tests in watch mode
npm run test:coverage # Generate test coverage report
npm run docs         # Generate TypeDoc documentation
npm run docs:serve   # Generate docs with watch mode
```

### Client Development
```bash
cd client
npm install          # Install dependencies
npm run dev          # Vite development server
npm run build        # Build for production
npm run type-check   # TypeScript type checking
npm test             # Run Vitest unit tests
npm run test:ui      # Run tests with UI interface
npm run test:coverage # Generate test coverage report
npm run docs         # Generate TypeDoc documentation
npm run docs:serve   # Generate docs with watch mode
```

## Testing Strategy

### Comprehensive Test Coverage
- **Server**: Jest + Supertest for API and storage adapter testing
- **Client**: Vitest + Vue Testing Library for component and logic testing  
- **Chaincode**: Jest + GalaChain test framework for contract methods
- **Integration**: E2E tests covering full game flow (`npm run test:e2e`)

### Key Test Areas
- `Chainstore` class: Storage adapter bridging boardgame.io with GalaChain
- TicTac contract methods: Match creation, state updates, player management
- Wallet connection: MetaMask integration and registration flow
- Game logic: Move validation, win detection, state management

### Documentation
- **TypeDoc**: Comprehensive API documentation for all public interfaces
- **Generated Docs**: Available via `npm run docs` in each codebase
- **Code Comments**: JSDoc comments on all public APIs and complex logic

## Important Implementation Details

### Chainstore Adapter (`server/src/chainstore.ts`)
- Implements boardgame.io's `Async` storage interface
- Serializes game state using GalaChain's DTO system
- Handles composite keys for state organization
- Manages transaction submission to chaincode

### GalaChain Integration
- Uses `@gala-chain/api` for DTO definitions and serialization
- Implements `GalaContract` with transaction decorators
- Enforces unique keys and signature verification
- State persistence uses composite key pattern

### Development Workflow
1. Start chaincode network: `cd tictac-chaincode && npm run network:start`
2. Start server: `cd server && npm run dev`
3. Start client: `cd client && npm run dev`
4. Access game at `http://localhost:5173`

## Code Conventions

- **TypeScript**: Strict typing throughout all components
- **DTOs**: Shared between server and chaincode for type safety
- **GalaChain Patterns**: Follow GalaChain decorator and DTO conventions
- **boardgame.io**: Standard patterns for game logic and state management