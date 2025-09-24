# 💎 Gem Store - GalaChain Integration Example

A minimal gem store application that demonstrates how to connect to GalaChain using MetaMask and execute transactions through wallet signing. This example focuses on teaching developers and AI systems the core patterns for GalaChain integration.

## 🎯 Learning Objectives

- Connect MetaMask wallet to GalaChain
- Check user registration status
- Auto-register new users
- Sign and submit burn transactions
- Handle transaction states and errors
- Display user balance and transaction history

## 🏗️ Architecture

- **Frontend**: Vue 3 + TypeScript + Vite
- **Backend**: NestJS + TypeScript
- **Database**: SQLite (for simplicity)
- **Wallet**: MetaMask + @gala-chain/connect

## 🚀 Quick Start

### Prerequisites

- Node.js 18+ 
- MetaMask browser extension
- GALA tokens in your MetaMask wallet

### Installation

1. **Install dependencies:**
   
   **Option A: Use the install script (recommended)**
   ```bash
   cd gem-store
   
   # On Windows
   install.bat
   
   # On Mac/Linux
   chmod +x install.sh
   ./install.sh
   ```
   
   **Option B: Manual installation**
   ```bash
   cd gem-store
   npm install
   cd frontend && npm install && cd ..
   cd backend && npm install && cd ..
   ```

2. **Set up environment variables:**
   ```bash
   # Copy the example environment file
   cp frontend/env.example frontend/.env
   
   # The default values should work for mainnet
   # No editing required unless you want to customize
   ```

3. **Start the development servers:**
   ```bash
   npm run dev
   ```

   This will start:
   - Frontend: http://localhost:3000
   - Backend: http://localhost:4000

### Usage

1. **Connect Wallet**: Click "Connect MetaMask" and approve the connection
2. **Auto-Registration**: New users are automatically registered with GalaChain
3. **Purchase Gems**: Select a gem package and burn GALA tokens to purchase gems
4. **View History**: Check your transaction history and gem balance

## 📁 Project Structure

```
gem-store/
├── frontend/                 # Vue.js frontend
│   ├── src/
│   │   ├── components/       # Vue components
│   │   │   ├── WalletConnect.vue
│   │   │   ├── GemStore.vue
│   │   │   └── TransactionHistory.vue
│   │   ├── App.vue          # Main app component
│   │   ├── main.ts          # App entry point
│   │   └── style.css        # Global styles
│   ├── package.json
│   └── vite.config.ts
├── backend/                  # NestJS backend
│   ├── src/
│   │   ├── entities/         # Database entities
│   │   ├── wallet/           # Wallet module
│   │   ├── transaction/      # Transaction module
│   │   ├── gem/              # Gem module
│   │   └── main.ts           # App entry point
│   └── package.json
└── package.json              # Root package.json
```

## 🔧 Configuration

### Environment Variables

Create `frontend/.env` with:

```env
# GalaChain API Endpoints
VITE_TOKEN_GATEWAY_API=https://gateway-mainnet.galachain.com/api/asset/token-contract
VITE_PUBLIC_KEY_GATEWAY_API=https://gateway-mainnet.galachain.com/api/asset/public-key-contract
VITE_CONNECT_API=https://api-galaswap.gala.com/galachain

# Project Configuration
VITE_PROJECT_ID=gem-store
VITE_PROJECT_API=http://localhost:4000
VITE_GEM_EXCHANGE_RATE=10
```

### Gem Packages

The default gem packages are:
- 10 gems for 1 GALA
- 50 gems for 5 GALA  
- 100 gems for 10 GALA
- 500 gems for 50 GALA

## 🔄 GalaChain Integration Flow

### 1. Wallet Connection
```typescript
// Connect to MetaMask
await metamaskClient.connect()
const address = metamaskClient.galaChainAddress

// Check registration
await checkRegistration(address)
// Auto-register if needed
await registerUser(metamaskClient)
```

### 2. Transaction Signing
```typescript
// Create burn transaction
const burnTokensDto = {
  owner: walletAddress,
  tokenInstances: [{
    quantity: galaAmount.toString(),
    tokenInstanceKey: {
      collection: "GALA",
      category: "Unit", 
      type: "none",
      additionalKey: "none",
      instance: "0"
    }
  }],
  uniqueKey: `gem-purchase-${Date.now()}`
}

// Sign with MetaMask
const signedTransaction = await metamaskClient.sign("BurnTokens", burnTokensDto)
```

### 3. Transaction Submission
```typescript
// Submit to GalaChain
const response = await fetch(`${TOKEN_GATEWAY_API}/BurnTokens`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify(signedTransaction)
})
```

## 🛠️ API Endpoints

### Wallet Management
- `GET /api/wallet/balance/:address` - Get user's gem balance
- `POST /api/wallet/connect` - Validate wallet connection

### Transactions  
- `POST /api/transactions/burn` - Process GALA burn for gems
- `GET /api/transactions/history/:address` - Get user transaction history

### Gems
- `GET /api/gems/packages` - Get available gem packages
- `GET /api/gems/exchange-rate` - Get current exchange rate

## 🧪 Testing

### Manual Testing

1. **Wallet Connection**: Verify MetaMask connection and auto-registration
2. **Transaction Flow**: Test gem purchases with different amounts
3. **Error Handling**: Test with insufficient balance, network errors
4. **History**: Verify transaction history displays correctly

### Test Scenarios

- ✅ Connect wallet successfully
- ✅ Auto-register new users
- ✅ Purchase gems with sufficient GALA
- ✅ Handle insufficient balance
- ✅ Display transaction history
- ✅ Handle network errors gracefully

## 🚨 Error Handling

The application handles common error scenarios:

- **MetaMask not installed**: Clear error message with installation link
- **User rejects connection**: Graceful handling with retry option
- **Insufficient balance**: Validation before transaction submission
- **Network errors**: Retry mechanisms and user feedback
- **Transaction failures**: Proper error logging and user notification

## 📚 Key Learning Points

1. **Wallet Integration**: Complete MetaMask connection flow
2. **User Registration**: Automatic GalaChain user registration
3. **Transaction Signing**: MetaMask transaction signing patterns
4. **Error Handling**: Comprehensive error handling strategies
5. **State Management**: Wallet and transaction state management
6. **API Design**: RESTful API design for blockchain integration

## 🔗 Related Resources

- [GalaChain Connect Library](https://github.com/GalaGames/gala-chain-connect)
- [MetaMask Documentation](https://docs.metamask.io/)
- [Vue.js Documentation](https://vuejs.org/)
- [NestJS Documentation](https://nestjs.com/)

## 📝 Notes

- This is a **learning example** - not production-ready
- Uses SQLite for simplicity - use PostgreSQL/MySQL for production
- No authentication beyond wallet connection
- Minimal error handling for educational purposes
- Exchange rates are hardcoded for simplicity

## 🤝 Contributing

This is an educational example. Feel free to:
- Report issues
- Suggest improvements
- Add more examples
- Improve documentation

## 📄 License

This project is for educational purposes. Please check GalaChain's licensing for production use.
