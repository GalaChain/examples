# Gem Store - GalaChain Wallet Integration Example

## Overview
A minimal gem store application that demonstrates how to connect to GalaChain using MetaMask and execute transactions through wallet signing. This example focuses on teaching developers and AI systems the core patterns for GalaChain integration.

## Core Learning Objectives
- Connect MetaMask wallet to GalaChain
- Check user registration status
- Auto-register new users
- Sign and submit burn transactions
- Handle transaction states and errors
- Display user balance and transaction history

## User Stories

### As a Developer Learning GalaChain Integration
- I want to see a complete wallet connection flow so I can understand the registration process
- I want to see transaction signing patterns so I can implement similar functionality
- I want to see error handling examples so I can build robust applications
- I want to see both frontend and backend code so I can understand the full stack

### As a User
- I want to connect my MetaMask wallet with one click
- I want to see my GALA balance clearly
- I want to purchase gems by burning GALA tokens
- I want to see transaction confirmations and status

## Technical Requirements

### Frontend (Vue.js)
- **Wallet Connection Component**
  - Connect/disconnect MetaMask wallet
  - Display wallet address and balance
  - Handle connection errors gracefully
  - Show loading states during connection

- **Gem Store Component**
  - Display available gem packages (e.g., 10 gems for 1 GALA, 50 gems for 5 GALA)
  - Show current gem balance
  - Purchase gems with burn transaction
  - Display transaction status and confirmations

- **Transaction History Component**
  - Show recent purchases
  - Display transaction IDs and timestamps
  - Show burn amounts and gem quantities

### Backend (NestJS)
- **Wallet Service**
  - Validate GalaChain addresses
  - Check user registration status
  - Handle user registration requests

- **Transaction Service**
  - Process burn transactions
  - Update user gem balances
  - Store transaction history
  - Validate transaction signatures

- **Gem Service**
  - Manage gem balances
  - Handle gem purchases
  - Calculate exchange rates

### Data Storage (Minimal)
- **User Table**
  - `id`, `walletAddress`, `gemBalance`, `createdAt`, `updatedAt`

- **Transaction Table**
  - `id`, `userWalletAddress`, `galaAmount`, `gemAmount`, `transactionId`, `status`, `createdAt`

## API Endpoints

### Wallet Management
- `POST /api/wallet/connect` - Validate wallet connection
- `GET /api/wallet/balance/:address` - Get user's gem balance
- `POST /api/wallet/register` - Register new user

### Transactions
- `POST /api/transactions/burn` - Process GALA burn for gems
- `GET /api/transactions/history/:address` - Get user transaction history

### Gems
- `GET /api/gems/packages` - Get available gem packages
- `POST /api/gems/purchase` - Purchase gems with GALA

## GalaChain Integration Points

### Wallet Connection Flow
1. User clicks "Connect Wallet"
2. MetaMask prompts for connection
3. Get GalaChain address (format: `eth|...`)
4. Check if user is registered via GalaChain API
5. Auto-register if not found
6. Display connected state and balance

### Transaction Flow
1. User selects gem package
2. Create burn transaction DTO
3. Sign transaction with MetaMask
4. Submit signed transaction to GalaChain
5. Wait for confirmation
6. Update local gem balance
7. Store transaction record

## Environment Configuration

### Option 1: Vite (Recommended for simplicity)
```env
# GalaChain API Endpoints
VITE_TOKEN_GATEWAY_API=https://gateway-mainnet.galachain.com/api/asset/token-contract
VITE_PUBLIC_KEY_GATEWAY_API=https://gateway-mainnet.galachain.com/api/asset/public-key-contract
VITE_CONNECT_API=https://api-galaswap.gala.com/galachain

# Project Configuration
VITE_PROJECT_ID=gem-store
VITE_PROJECT_API=http://localhost:4000
VITE_GEM_EXCHANGE_RATE=10  # 1 GALA = 10 gems
```

### Option 2: Webpack/Other Build Tools
```env
# GalaChain API Endpoints
REACT_APP_TOKEN_GATEWAY_API=https://gateway-mainnet.galachain.com/api/asset/token-contract
REACT_APP_PUBLIC_KEY_GATEWAY_API=https://gateway-mainnet.galachain.com/api/asset/public-key-contract
REACT_APP_CONNECT_API=https://api-galaswap.gala.com/galachain

# Project Configuration
REACT_APP_PROJECT_ID=gem-store
REACT_APP_PROJECT_API=http://localhost:4000
REACT_APP_GEM_EXCHANGE_RATE=10
```

### Option 3: Runtime Configuration (No build tool env vars)
```typescript
// config.ts
export const config = {
  tokenGatewayApi: 'https://gateway-mainnet.galachain.com/api/asset/token-contract',
  publicKeyGatewayApi: 'https://gateway-mainnet.galachain.com/api/asset/public-key-contract',
  connectApi: 'https://api-galaswap.gala.com/galachain',
  projectId: 'gem-store',
  projectApi: 'http://localhost:4000',
  gemExchangeRate: 10
}
```

### TypeScript Interfaces

**For Vite:**
```typescript
// env.d.ts
interface ImportMetaEnv {
  readonly VITE_TOKEN_GATEWAY_API: string
  readonly VITE_PUBLIC_KEY_GATEWAY_API: string
  readonly VITE_CONNECT_API: string
  readonly VITE_PROJECT_ID: string
  readonly VITE_PROJECT_API: string
  readonly VITE_GEM_EXCHANGE_RATE?: string
}
```

**For React/Webpack:**
```typescript
// env.d.ts
declare namespace NodeJS {
  interface ProcessEnv {
    readonly REACT_APP_TOKEN_GATEWAY_API: string
    readonly REACT_APP_PUBLIC_KEY_GATEWAY_API: string
    readonly REACT_APP_CONNECT_API: string
    readonly REACT_APP_PROJECT_ID: string
    readonly REACT_APP_PROJECT_API: string
    readonly REACT_APP_GEM_EXCHANGE_RATE?: string
  }
}
```

## Success Criteria
- [ ] Users can connect MetaMask wallet successfully
- [ ] New users are automatically registered with GalaChain
- [ ] Users can burn GALA to purchase gems
- [ ] Transaction history is displayed correctly
- [ ] Error handling works for all failure scenarios
- [ ] Code is well-documented with learning comments
- [ ] Example can be run locally with minimal setup

## Out of Scope (Keep Minimal)
- User authentication beyond wallet connection
- Complex gem mechanics or game features
- Advanced transaction types beyond burning
- Production-grade security features
- Complex UI/UX beyond basic functionality
- Database migrations or complex data relationships

## Technical Stack
- **Frontend**: Vue 3 + TypeScript + Build Tool (Vite recommended, but Webpack/others work)
- **Backend**: NestJS + TypeScript
- **Database**: SQLite (for simplicity)
- **Wallet**: MetaMask + @gala-chain/connect
- **Styling**: Basic CSS (no complex frameworks)

**Note**: Vite is recommended for its simplicity and fast development experience, but the GalaChain integration patterns work with any modern build tool or even vanilla JavaScript.

## File Structure
```
gem-store/
├── frontend/
│   ├── src/
│   │   ├── components/
│   │   │   ├── WalletConnect.vue
│   │   │   ├── GemStore.vue
│   │   │   └── TransactionHistory.vue
│   │   ├── services/
│   │   │   └── wallet.service.ts
│   │   └── main.ts
│   └── package.json
├── backend/
│   ├── src/
│   │   ├── wallet/
│   │   ├── transactions/
│   │   ├── gems/
│   │   └── main.ts
│   └── package.json
└── README.md
```

This example should serve as a clear, educational reference for anyone wanting to understand GalaChain wallet integration patterns. 