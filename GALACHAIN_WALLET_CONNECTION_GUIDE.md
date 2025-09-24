# GalaChain MetaMask Wallet Connection Guide

This guide provides a comprehensive overview of how to integrate MetaMask wallet connection and GalaChain transaction signing into your web application. Based on analysis of the GalaChain examples codebase, this document covers best practices, code patterns, and implementation details for building production-ready blockchain applications.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Environment Configuration](#environment-configuration)
4. [Wallet Connection](#wallet-connection)
5. [User Registration](#user-registration)
6. [Transaction Signing](#transaction-signing)
7. [Common Patterns](#common-patterns)
8. [Best Practices](#best-practices)
9. [Error Handling](#error-handling)
10. [Complete Examples](#complete-examples)

## Prerequisites

- Node.js environment
- MetaMask wallet browser extension
- Basic understanding of TypeScript/JavaScript
- GalaChain Connect library
- HTTP client library (axios or fetch)

## Installation

Install the required GalaChain Connect library:

```bash
npm install @gala-chain/connect@^2.3.4
```

For React applications, you may also need:

```bash
npm install @gala-chain/api
```

## Environment Configuration

Create a `.env` file in your project root with the following variables:

```env
# GalaChain API Endpoints (Use descriptive names for clarity)
VITE_TOKEN_GATEWAY_API=https://gateway-mainnet.galachain.com/api/asset/token-contract
VITE_PUBLIC_KEY_GATEWAY_API=https://gateway-mainnet.galachain.com/api/asset/public-key-contract
VITE_CONNECT_API=https://api-galaswap.gala.com/galachain

# Project Configuration
VITE_PROJECT_ID=your-project-id
VITE_PROJECT_API=http://localhost:4000
```

Define TypeScript interfaces for environment variables:

```typescript
// env.d.ts
interface ImportMetaEnv {
  readonly VITE_TOKEN_GATEWAY_API: string
  readonly VITE_PUBLIC_KEY_GATEWAY_API: string
  readonly VITE_CONNECT_API: string
  readonly VITE_PROJECT_ID: string
  readonly VITE_PROJECT_API: string
}
```

## Wallet Connection

### Service-Based Architecture (Recommended)

A clean service-based architecture separates wallet logic from UI components, making your code more maintainable and testable:

```typescript
// services/walletService.ts
import { BrowserConnectClient } from '@gala-chain/connect'

export interface ConnectionResult {
  address: string
  provider: any
}

export const walletService = {
  client: null as BrowserConnectClient | null,
  
  async connect(): Promise<ConnectionResult> {
    this.client = new BrowserConnectClient()
    const connected = await this.client.connect()
    
    if (connected && this.client.galaChainAddress) {
      return {
        address: this.client.galaChainAddress,
        provider: this.client
      }
    }
    throw new Error('Failed to connect wallet')
  },
  
  async signMessage(message: string) {
    if (!this.client) throw new Error('Wallet not connected')
    
    const timestamp = Date.now()
    const fullMessage = `Sign this message to authenticate:\n${message}\nTimestamp: ${timestamp}`
    
    const signature = await this.client.personalSign(fullMessage)
    return { signature, timestamp }
  },
  
  disconnect() {
    if (this.client) {
      this.client.disconnect()
      this.client = null
    }
  }
}
```

### Basic Connection Setup

```typescript
import { walletService } from './services/walletService'

// Check if MetaMask is available
const metamaskSupport = !!window.ethereum

// Connection state
const [isConnected, setIsConnected] = useState(false)
const [walletAddress, setWalletAddress] = useState('')
const [isConnecting, setIsConnecting] = useState(false)
```

### Connect Wallet Function

```typescript
async function connectWallet() {
  if (!metamaskSupport) {
    throw new Error('MetaMask not detected')
  }

  try {
    setIsConnecting(true)
    
    // Connect to MetaMask using wallet service
    const connectionResult = await walletService.connect()
    
    if (!connectionResult || !connectionResult.address) {
      throw new Error('Failed to connect to MetaMask')
    }
    
    const address = connectionResult.address
    setWalletAddress(address)
    setIsConnected(true)
    
    // Check if user is registered (optional for simpler implementations)
    // await checkRegistration(address)
    
  } catch (error) {
    console.error('Error connecting wallet:', error)
    throw error
  } finally {
    setIsConnecting(false)
  }
}
```

### Vue.js Implementation (Production Pattern)

```vue
<template>
  <div class="card">
    <h2>Connect Your Wallet</h2>
    
    <div v-if="!isConnected" class="connect-section">
      <p class="mb-3">
        Connect your MetaMask wallet to start using GALA tokens.
      </p>
      
      <button 
        @click="connectWallet" 
        :disabled="isConnecting"
        class="btn"
      >
        <span v-if="isConnecting" class="loading"></span>
        {{ isConnecting ? 'Connecting...' : 'Connect MetaMask' }}
      </button>
      
      <div v-if="error" class="status error mt-3">
        {{ error }}
      </div>
    </div>
    
    <div v-else class="wallet-connected">
      <div class="wallet-info">
        <h3>✅ Wallet Connected</h3>
        <p class="wallet-address">{{ walletAddress }}</p>
        <p class="balance">Balance: {{ galaBalance }} GALA</p>
      </div>
      
      <div class="status success">
        <strong>Ready to transact!</strong> You can now use GALA tokens for transactions.
      </div>
      
      <div class="mt-3">
        <button @click="disconnectWallet" class="btn btn-danger ml-2">
          Disconnect
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, inject } from 'vue'
import { walletService } from '../services/walletService'

// Props and emits
const emit = defineEmits(['wallet-connected', 'wallet-disconnected'])

// Injected wallet state (from parent component)
const walletAddress = inject('walletAddress')
const isWalletConnected = inject('isWalletConnected')

// Local state
const isConnected = ref(false)
const isConnecting = ref(false)
const error = ref('')
const galaBalance = ref(0)

// Check if MetaMask is available
const metamaskSupport = !!window.ethereum

// Connect wallet function
const connectWallet = async () => {
  if (!metamaskSupport) {
    error.value = 'MetaMask not detected. Please install MetaMask extension.'
    return
  }

  try {
    isConnecting.value = true
    error.value = ''
    
    // Connect to MetaMask using wallet service
    const connectionResult = await walletService.connect()
    
    if (!connectionResult || !connectionResult.address) {
      throw new Error('Failed to connect to MetaMask')
    }
    
    const address = connectionResult.address
    
    // Update state
    walletAddress.value = address
    isConnected.value = true
    isWalletConnected.value = true
    
    // Fetch balance
    await fetchBalance(address)
    
    // Emit event
    emit('wallet-connected', address)
    
  } catch (err) {
    console.error('Error connecting wallet:', err)
    error.value = err instanceof Error ? err.message : 'Failed to connect wallet'
  } finally {
    isConnecting.value = false
  }
}

// Fetch GALA balance
const fetchBalance = async (address: string) => {
  try {
    const balanceDto = {
      owner: address,
      collection: "GALA",
      category: "Unit",
      type: "none",
      additionalKey: "none",
      instance: "0"
    }

    const response = await fetch(
      `${import.meta.env.VITE_TOKEN_GATEWAY_API}/FetchBalances`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(balanceDto)
      }
    )

    if (response.ok) {
      const result = await response.json()
      if (result.Data && result.Data.length > 0) {
        galaBalance.value = parseFloat(result.Data[0].quantity)
      }
    }
  } catch (err) {
    console.error('Error fetching balance:', err)
  }
}

// Disconnect wallet
const disconnectWallet = async () => {
  try {
    walletService.disconnect()
    isConnected.value = false
    isWalletConnected.value = false
    walletAddress.value = ''
    galaBalance.value = 0
    error.value = ''
    emit('wallet-disconnected')
  } catch (err) {
    console.error('Error disconnecting wallet:', err)
  }
}
</script>
```

### React Implementation (Updated Pattern)

```tsx
import { useState, useEffect } from 'react'
import { walletService } from './services/walletService'

export default function WalletConnect({ onAddressChange }: { onAddressChange: (address: string) => void }) {
  const [account, setAccount] = useState<string>('')
  const [isConnecting, setIsConnecting] = useState(false)
  const [error, setError] = useState<string>('')
  const [galaBalance, setGalaBalance] = useState<number>(0)

  const connectWallet = async () => {
    if (!window.ethereum) {
      setError('MetaMask not detected. Please install MetaMask extension.')
      return
    }

    try {
      setIsConnecting(true)
      setError('')
      
      const connectionResult = await walletService.connect()
      
      if (!connectionResult || !connectionResult.address) {
        throw new Error('Failed to connect to MetaMask')
      }
      
      const address = connectionResult.address
      setAccount(address)
      onAddressChange(address)
      
      // Fetch balance
      await fetchBalance(address)
      
    } catch (error) {
      console.error('Failed to connect wallet:', error)
      setError(error instanceof Error ? error.message : 'Failed to connect wallet')
    } finally {
      setIsConnecting(false)
    }
  }

  const fetchBalance = async (address: string) => {
    try {
      const balanceDto = {
        owner: address,
        collection: "GALA",
        category: "Unit",
        type: "none",
        additionalKey: "none",
        instance: "0"
      }

      const response = await fetch(
        `${import.meta.env.VITE_TOKEN_GATEWAY_API}/FetchBalances`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(balanceDto)
        }
      )

      if (response.ok) {
        const result = await response.json()
        if (result.Data && result.Data.length > 0) {
          setGalaBalance(parseFloat(result.Data[0].quantity))
        }
      }
    } catch (err) {
      console.error('Error fetching balance:', err)
    }
  }

  const disconnectWallet = () => {
    walletService.disconnect()
    setAccount('')
    setGalaBalance(0)
    setError('')
    onAddressChange('')
  }

  return (
    <div>
      {account ? (
        <div>
          <h3>✅ Wallet Connected</h3>
          <p>Connected Account: {account}</p>
          <p>Balance: {galaBalance} GALA</p>
          <button onClick={disconnectWallet}>
            Disconnect
          </button>
        </div>
      ) : (
        <div>
          <button onClick={connectWallet} disabled={isConnecting}>
            {isConnecting ? 'Connecting...' : 'Connect Wallet'}
          </button>
          {error && <p style={{ color: 'red' }}>{error}</p>}
        </div>
      )}
    </div>
  )
}
```

## User Registration

### Check Registration Status

```typescript
async function checkRegistration(walletAddress: string) {
  try {
    const response = await fetch(
      `${import.meta.env.VITE_PUBLIC_KEY_GATEWAY_API}/GetPublicKey`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ user: walletAddress })
      }
    )
    
    if (!response.ok) {
      throw new Error('User not registered')
    }
    
    return true
  } catch (error) {
    console.log('User is not registered:', error)
    return false
  }
}
```

**Best Practice**: Registration checking can be optional for simpler implementations. Many applications can work without explicit registration checks, as GalaChain handles user registration automatically when needed.

### Register New User

```typescript
async function registerUser(metamaskClient: BrowserConnectClient) {
  try {
    // Get public key from MetaMask
    const publicKey = await metamaskClient.getPublicKey()
    
    // Register with GalaChain
    const response = await fetch(
      `${import.meta.env.VITE_CONNECT_API}/CreateHeadlessWallet`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ publicKey: publicKey.publicKey })
      }
    )
    
    if (!response.ok) {
      throw new Error('Failed to register user')
    }
    
    return true
  } catch (error) {
    console.error('Error registering user:', error)
    throw error
  }
}
```

### Complete Registration Flow

```typescript
async function connectWallet() {
  try {
    const connectionResult = await walletService.connect()
    const address = connectionResult.address
    
    // Check registration (optional for simpler implementations)
    try {
      await checkRegistration(address)
    } catch (e) {
      console.log(`Registration check failed: ${e}. Attempting to register user: ${address}`)
      await registerUser(walletService.client)
    }
    
    setWalletAddress(address)
    setIsConnected(true)
  } catch (err) {
    console.error('Error connecting wallet:', err)
  }
}
```

## Transaction Signing

### Token Burn Transaction (Production Pattern)

```typescript
async function burnTokens(amount: number, walletAddress: string) {
  try {
    const burnTokensDto = {
      owner: walletAddress,
      tokenInstances: [{
        quantity: amount.toString(),
        tokenInstanceKey: {
          collection: "GALA",
          category: "Unit",
          type: "none",
          additionalKey: "none",
          instance: "0"
        }
      }],
      uniqueKey: `burn-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
    }

    // Sign the transaction with MetaMask
    if (!walletService.client) throw new Error('Wallet not connected')
    const signedBurnDto = await walletService.client.sign("BurnTokens", burnTokensDto)
    
    // Submit to backend (recommended pattern for production apps)
    const response = await fetch('/api/transactions/burn', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        signedTransaction: signedBurnDto,
        amount: amount,
        walletAddress: walletAddress
      })
    })

    if (!response.ok) {
      throw new Error('Failed to process transaction')
    }

    const result = await response.json()
    return result
  } catch (error) {
    console.error('Error burning tokens:', error)
    throw error
  }
}
```

**Key Production Improvements:**
1. **More unique transaction keys** using timestamp + random string
2. **Backend submission pattern** instead of direct GalaChain API calls
3. **Structured transaction data** with proper validation
4. **Better error handling** with specific error messages

### Token Transfer Transaction

```typescript
async function transferTokens(
  from: string,
  to: string,
  amount: number,
  walletService: any
) {
  try {
    const transferTokensDto = {
      from: from,
      to: to,
      quantity: amount.toString(),
      tokenInstance: {
        collection: "GALA",
        category: "Unit",
        type: "none",
        additionalKey: "none",
        instance: "0"
      },
      uniqueKey: `transfer-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
    }

    // Sign the transaction
    if (!walletService.client) throw new Error('Wallet not connected')
    const signedTransferDto = await walletService.client.sign("TransferTokens", transferTokensDto)

    // Submit to GalaChain
    const response = await fetch(
      `${import.meta.env.VITE_TOKEN_GATEWAY_API}/TransferToken`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(signedTransferDto)
      }
    )

    if (!response.ok) {
      throw new Error('Failed to transfer tokens')
    }

    const result = await response.json()
    return result
  } catch (error) {
    console.error('Error transferring tokens:', error)
    throw error
  }
}
```

### Check Token Balance

```typescript
async function fetchBalance(walletAddress: string) {
  try {
    const balanceDto = {
      owner: walletAddress,
      collection: "GALA",
      category: "Unit",
      type: "none",
      additionalKey: "none",
      instance: "0"
    }

    const response = await fetch(
      `${import.meta.env.VITE_TOKEN_GATEWAY_API}/FetchBalances`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(balanceDto)
      }
    )

    if (!response.ok) {
      throw new Error('Failed to fetch balance')
    }

    const result = await response.json()
    let balance = 0

    if (result.Data && result.Data.length > 0) {
      balance = parseFloat(result.Data[0].quantity)
      
      // Calculate locked balance
      const lockedBalance = result.Data[0].lockedHolds?.reduce(
        (acc: number, hold: any) => acc + parseFloat(hold.quantity), 0
      ) || 0
      
      return { balance, lockedBalance }
    }

    return { balance: 0, lockedBalance: 0 }
  } catch (error) {
    console.error('Error fetching balance:', error)
    throw error
  }
}
```

## Backend Integration Patterns

### Backend Transaction Processing (Production Pattern)

Production applications should implement a clean separation between frontend wallet operations and backend transaction processing:

```typescript
// Backend: Transaction Service
@Injectable()
export class TransactionService {
  async processBurnTransaction(
    signedTransaction: any,
    gemAmount: number,
    galaAmount: number,
    walletAddress: string
  ): Promise<{ transactionId: string }> {
    try {
      // Validate wallet address
      const isValid = await this.walletService.validateWalletAddress(walletAddress);
      if (!isValid) {
        throw new HttpException('Invalid wallet address', HttpStatus.BAD_REQUEST);
      }

      // Submit the signed transaction to GalaChain
      const galaChainResponse = await this.submitToGalaChain(signedTransaction);
      
      if (!galaChainResponse.success) {
        throw new HttpException('GalaChain transaction failed', HttpStatus.BAD_REQUEST);
      }

      // Create transaction record
      const transaction = this.transactionRepository.create({
        userWalletAddress: walletAddress,
        galaAmount,
        gemAmount,
        transactionId: galaChainResponse.transactionId,
        status: 'completed'
      });

      await this.transactionRepository.save(transaction);

      // Add gems to user's balance
      await this.walletService.addGems(walletAddress, gemAmount);

      return { transactionId: galaChainResponse.transactionId };

    } catch (error) {
      // Create failed transaction record
      const transaction = this.transactionRepository.create({
        userWalletAddress: walletAddress,
        galaAmount,
        gemAmount,
        transactionId: `failed-${Date.now()}`,
        status: 'failed'
      });

      await this.transactionRepository.save(transaction);

      throw new HttpException(
        error.message || 'Transaction processing failed',
        error.status || HttpStatus.INTERNAL_SERVER_ERROR
      );
    }
  }

  private async submitToGalaChain(signedTransaction: any): Promise<{
    success: boolean;
    transactionId: string;
  }> {
    try {
      const response = await fetch(
        `${process.env.TOKEN_GATEWAY_API}/BurnTokens`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(signedTransaction)
        }
      );

      if (!response.ok) {
        throw new Error(`GalaChain API error: ${response.status}`);
      }

      const result = await response.json();
      
      return {
        success: true,
        transactionId: result.transactionId || `tx-${Date.now()}`
      };

    } catch (error) {
      console.error('GalaChain submission error:', error);
      return {
        success: false,
        transactionId: `failed-${Date.now()}`
      };
    }
  }
}
```

### Wallet Address Validation

```typescript
// Backend: Wallet Service
@Injectable()
export class WalletService {
  async validateWalletAddress(walletAddress: string): Promise<boolean> {
    // Basic validation for wallet address format
    // Accept both GalaChain format (eth|, client|) and standard Ethereum format (0x)
    return walletAddress.startsWith('eth|') || 
           walletAddress.startsWith('client|') || 
           walletAddress.startsWith('0x');
  }

  async findOrCreateUser(walletAddress: string): Promise<User> {
    let user = await this.userRepository.findOne({
      where: { walletAddress }
    });

    if (!user) {
      user = this.userRepository.create({
        walletAddress,
        gemBalance: 0
      });
      await this.userRepository.save(user);
    }

    return user;
  }
}
```

### API Controller Pattern

```typescript
// Backend: Transaction Controller
@Controller('transactions')
export class TransactionController {
  constructor(private readonly transactionService: TransactionService) {}

  @Post('burn')
  async burnTokens(@Body() body: BurnTransactionDto) {
    try {
      const { signedTransaction, gemAmount, galaAmount, walletAddress } = body;

      if (gemAmount <= 0 || galaAmount <= 0) {
        throw new HttpException('Amounts must be positive', HttpStatus.BAD_REQUEST);
      }

      const result = await this.transactionService.processBurnTransaction(
        signedTransaction,
        gemAmount,
        galaAmount,
        walletAddress
      );

      return {
        success: true,
        message: 'Transaction processed successfully',
        ...result
      };

    } catch (error) {
      throw new HttpException(
        error.message || 'Failed to process burn transaction',
        error.status || HttpStatus.INTERNAL_SERVER_ERROR
      );
    }
  }
}
```

## Common Patterns

### Address Format Conversion

GalaChain addresses are typically prefixed with `eth|` for Ethereum-based wallets:

```typescript
function formatGalaChainAddress(ethereumAddress: string): string {
  if (ethereumAddress.startsWith('0x')) {
    return `eth|${ethereumAddress.slice(2)}`
  }
  return ethereumAddress
}

// Usage
const galaAddress = formatGalaChainAddress(walletService.client.getWalletAddress)
```

**Best Practice**: The `BrowserConnectClient` automatically handles address formatting, so manual conversion is typically not needed.

### Transaction State Management

```typescript
interface TransactionState {
  isProcessing: boolean
  error: string | null
  success: string | null
}

function useTransaction() {
  const [state, setState] = useState<TransactionState>({
    isProcessing: false,
    error: null,
    success: null
  })

  const executeTransaction = async (transactionFn: () => Promise<any>) => {
    setState({ isProcessing: true, error: null, success: null })
    
    try {
      const result = await transactionFn()
      setState({ isProcessing: false, error: null, success: 'Transaction successful' })
      return result
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      setState({ isProcessing: false, error: errorMessage, success: null })
      throw error
    }
  }

  return { ...state, executeTransaction }
}
```

### Auto-connect on Page Load

```typescript
useEffect(() => {
  const checkExistingConnection = async () => {
    if (!metamaskClient) return

    try {
      const connected = await metamaskClient.connect()
      if (connected) {
        const address = metamaskClient.galaChainAddress
        setAccount(address)
        onAddressChange(address)
        await checkRegistration(address)
      }
    } catch (error) {
      console.error('Failed to check connection:', error)
    }
  }

  checkExistingConnection()
}, [metamaskClient, onAddressChange])
```

## Best Practices

### 1. Error Handling

Always implement comprehensive error handling:

```typescript
try {
  await metamaskClient.connect()
} catch (error) {
  if (error.code === 4001) {
    // User rejected the connection request
    setError('User rejected wallet connection')
  } else if (error.code === -32002) {
    // Request already pending
    setError('Connection request already pending')
  } else {
    setError('Failed to connect wallet')
  }
}
```

### 2. Loading States

Provide clear feedback during async operations:

```typescript
const [isConnecting, setIsConnecting] = useState(false)
const [isProcessing, setIsProcessing] = useState(false)

// In UI
<button disabled={isConnecting || isProcessing}>
  {isConnecting ? 'Connecting...' : isProcessing ? 'Processing...' : 'Connect Wallet'}
</button>
```

### 3. Input Validation

Validate user inputs before transactions:

```typescript
const isValidAmount = (amount: number): boolean => {
  return amount > 0 && !isNaN(amount) && isFinite(amount)
}

const isValidAddress = (address: string): boolean => {
  return address.startsWith('eth|') || address.startsWith('client|')
}
```

### 4. Unique Transaction Keys

Always use unique keys for transactions to prevent replay attacks:

```typescript
const uniqueKey = `transaction-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
```

### 5. Environment-Specific Configuration

Use different API endpoints for development and production:

```typescript
const getApiUrl = () => {
  if (import.meta.env.DEV) {
    return 'http://localhost:4000'
  }
  return import.meta.env.VITE_TOKEN_GATEWAY_API
}
```

### 6. Service-Based Architecture (Production Pattern)

Production-ready applications should follow these key architectural patterns:

- **Separation of Concerns**: Wallet logic is separated into a dedicated service
- **Centralized State Management**: Wallet state is managed at the app level and provided to components
- **Backend Integration**: Transactions are processed through a backend API rather than direct GalaChain calls
- **Error Handling**: Comprehensive error handling with user-friendly messages
- **Transaction Recording**: All transactions are recorded in a local database for history tracking

## Error Handling

### Common Error Scenarios

```typescript
class WalletError extends Error {
  constructor(message: string, public code?: string) {
    super(message)
    this.name = 'WalletError'
  }
}

function handleWalletError(error: any): string {
  switch (error.code) {
    case 4001:
      return 'User rejected the request'
    case -32002:
      return 'Request already pending'
    case -32602:
      return 'Invalid parameters'
    case -32603:
      return 'Internal error'
    default:
      return error.message || 'Unknown wallet error'
  }
}
```

### Network Error Handling

```typescript
async function makeApiRequest(url: string, options: RequestInit) {
  try {
    const response = await fetch(url, options)
    
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }
    
    return await response.json()
  } catch (error) {
    if (error instanceof TypeError) {
      throw new Error('Network error - please check your connection')
    }
    throw error
  }
}
```

## Complete Examples

### Complete Production Implementation Pattern

Here's a comprehensive pattern for production-ready GalaChain wallet integration:

#### Frontend App Structure (Vue.js)

```vue
<!-- App.vue - Main application with wallet state management -->
<template>
  <div id="app">
    <header class="header">
      <h1>🚀 GalaChain App</h1>
      <p>GalaChain Wallet Integration Example</p>
    </header>

    <div class="container">
      <nav class="nav">
        <router-link to="/" :class="{ active: $route.path === '/' }">
          Connect Wallet
        </router-link>
        <router-link 
          to="/transactions" 
          :class="{ active: $route.path === '/transactions' }"
          v-if="isWalletConnected"
        >
          Transactions
        </router-link>
        <router-link 
          to="/history" 
          :class="{ active: $route.path === '/history' }"
          v-if="isWalletConnected"
        >
          Transaction History
        </router-link>
      </nav>

      <router-view 
        :wallet-address="walletAddress"
        :is-connected="isWalletConnected"
        @wallet-connected="handleWalletConnected"
        @wallet-disconnected="handleWalletDisconnected"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, provide } from 'vue'
import { walletService } from './services/walletService'

// Wallet state
const isWalletConnected = ref(false)
const walletAddress = ref('')

// Provide wallet state to child components
provide('walletAddress', walletAddress)
provide('isWalletConnected', isWalletConnected)
provide('walletService', walletService)

// Event handlers
const handleWalletConnected = (address: string) => {
  walletAddress.value = address
  isWalletConnected.value = true
}

const handleWalletDisconnected = () => {
  walletAddress.value = ''
  isWalletConnected.value = false
}
</script>
```

#### Backend API Structure (NestJS)

```typescript
// Backend: Complete transaction flow
@Controller('transactions')
export class TransactionController {
  constructor(private readonly transactionService: TransactionService) {}

  @Post('burn')
  async burnTokens(@Body() body: BurnTransactionDto) {
    try {
      const { signedTransaction, gemAmount, galaAmount, walletAddress } = body;

      if (gemAmount <= 0 || galaAmount <= 0) {
        throw new HttpException('Amounts must be positive', HttpStatus.BAD_REQUEST);
      }

      const result = await this.transactionService.processBurnTransaction(
        signedTransaction,
        gemAmount,
        galaAmount,
        walletAddress
      );

      return {
        success: true,
        message: 'Transaction processed successfully',
        ...result
      };

    } catch (error) {
      throw new HttpException(
        error.message || 'Failed to process burn transaction',
        error.status || HttpStatus.INTERNAL_SERVER_ERROR
      );
    }
  }

  @Get('history/:address')
  async getTransactionHistory(@Param('address') address: string) {
    try {
      const result = await this.transactionService.getTransactionHistory(address);
      return {
        success: true,
        ...result
      };
    } catch (error) {
      throw new HttpException(
        error.message || 'Failed to fetch transaction history',
        error.status || HttpStatus.INTERNAL_SERVER_ERROR
      );
    }
  }
}
```

#### Database Entities

```typescript
// User Entity
@Entity('users')
export class User {
  @PrimaryGeneratedColumn()
  id: number;

  @Column({ unique: true })
  walletAddress: string;

  @Column({ default: 0 })
  tokenBalance: number;

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;

  @OneToMany(() => Transaction, transaction => transaction.user)
  transactions: Transaction[];
}

// Transaction Entity
@Entity('transactions')
export class Transaction {
  @PrimaryGeneratedColumn()
  id: number;

  @Column()
  userWalletAddress: string;

  @Column('decimal', { precision: 10, scale: 6 })
  galaAmount: number;

  @Column('int')
  tokenAmount: number;

  @Column()
  transactionId: string;

  @Column({
    type: 'varchar',
    default: 'pending'
  })
  status: string;

  @CreateDateColumn()
  createdAt: Date;

  @ManyToOne(() => User, user => user.transactions)
  @JoinColumn({ name: 'userWalletAddress', referencedColumnName: 'walletAddress' })
  user: User;
}
```

### Vue.js Complete Implementation

```vue
<template>
  <div class="wallet-app">
    <div v-if="!isConnected" class="connect-section">
      <button @click="connectWallet" :disabled="isConnecting" class="connect-btn">
        {{ isConnecting ? 'Connecting...' : 'Connect MetaMask' }}
      </button>
      <p v-if="error" class="error">{{ error }}</p>
    </div>
    
    <div v-else class="wallet-connected">
      <h2>Wallet Connected</h2>
      <p>Address: {{ walletAddress }}</p>
      <p>Balance: {{ balance }} GALA</p>
      
      <div class="actions">
        <BurnGala 
          :wallet-address="walletAddress" 
          :metamask-client="metamaskClient" 
        />
        <TransferGala 
          :wallet-address="walletAddress" 
          :metamask-client="metamaskClient" 
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { BrowserConnectClient } from '@gala-chain/connect'
import BurnGala from './components/BurnGala.vue'
import TransferGala from './components/TransferGala.vue'

const metamaskClient = new BrowserConnectClient()
const isConnected = ref(false)
const walletAddress = ref('')
const balance = ref(0)
const isConnecting = ref(false)
const error = ref('')

const connectWallet = async () => {
  try {
    isConnecting.value = true
    error.value = ''
    
    await metamaskClient.connect()
    walletAddress.value = metamaskClient.galaChainAddress
    
    // Check registration
    try {
      await checkRegistration(walletAddress.value)
    } catch (e) {
      console.log('User not registered, registering...')
      await registerUser(metamaskClient)
    }
    
    isConnected.value = true
    await fetchBalance()
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to connect wallet'
  } finally {
    isConnecting.value = false
  }
}

const checkRegistration = async (address: string) => {
  const response = await fetch(
    `${import.meta.env.PUBLIC_KEY_GATEWAY_API}/GetPublicKey`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ user: address })
    }
  )
  if (!response.ok) throw new Error('User not registered')
}

const registerUser = async (client: BrowserConnectClient) => {
  const publicKey = await client.getPublicKey()
  await fetch(`${import.meta.env.CONNECT_API}/CreateHeadlessWallet`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ publicKey: publicKey.publicKey })
  })
}

const fetchBalance = async () => {
  try {
    const balanceDto = {
      owner: walletAddress.value,
      collection: "GALA",
      category: "Unit",
      type: "none",
      additionalKey: "none",
      instance: "0"
    }

    const response = await fetch(
      `${import.meta.env.TOKEN_GATEWAY_API}/FetchBalances`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(balanceDto)
      }
    )

    if (response.ok) {
      const result = await response.json()
      if (result.Data && result.Data.length > 0) {
        balance.value = parseFloat(result.Data[0].quantity)
      }
    }
  } catch (err) {
    console.error('Error fetching balance:', err)
  }
}

onMounted(() => {
  // Check for existing connection
  if (window.ethereum) {
    connectWallet()
  }
})
</script>
```

### React Complete Implementation

```tsx
import React, { useState, useEffect } from 'react'
import { BrowserConnectClient } from '@gala-chain/connect'

interface WalletState {
  isConnected: boolean
  address: string
  balance: number
  isConnecting: boolean
  error: string | null
}

export default function WalletApp() {
  const [walletState, setWalletState] = useState<WalletState>({
    isConnected: false,
    address: '',
    balance: 0,
    isConnecting: false,
    error: null
  })
  
  const [metamaskClient, setMetamaskClient] = useState<BrowserConnectClient | null>(null)

  useEffect(() => {
    setMetamaskClient(new BrowserConnectClient())
  }, [])

  const connectWallet = async () => {
    if (!metamaskClient) return

    setWalletState(prev => ({ ...prev, isConnecting: true, error: null }))

    try {
      await metamaskClient.connect()
      const address = metamaskClient.galaChainAddress

      // Check registration
      try {
        await checkRegistration(address)
      } catch (e) {
        console.log('User not registered, registering...')
        await registerUser(metamaskClient)
      }

      setWalletState(prev => ({
        ...prev,
        isConnected: true,
        address,
        isConnecting: false
      }))

      await fetchBalance(address)
    } catch (error) {
      setWalletState(prev => ({
        ...prev,
        isConnecting: false,
        error: error instanceof Error ? error.message : 'Failed to connect wallet'
      }))
    }
  }

  const checkRegistration = async (address: string) => {
    const response = await fetch(
      `${import.meta.env.PUBLIC_KEY_GATEWAY_API}/GetPublicKey`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ user: address })
      }
    )
    if (!response.ok) throw new Error('User not registered')
  }

  const registerUser = async (client: BrowserConnectClient) => {
    const publicKey = await client.getPublicKey()
    await fetch(`${import.meta.env.CONNECT_API}/CreateHeadlessWallet`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ publicKey: publicKey.publicKey })
    })
  }

  const fetchBalance = async (address: string) => {
    try {
      const balanceDto = {
        owner: address,
        collection: "GALA",
        category: "Unit",
        type: "none",
        additionalKey: "none",
        instance: "0"
      }

      const response = await fetch(
        `${import.meta.env.TOKEN_GATEWAY_API}/FetchBalances`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(balanceDto)
        }
      )

      if (response.ok) {
        const result = await response.json()
        if (result.Data && result.Data.length > 0) {
          setWalletState(prev => ({
            ...prev,
            balance: parseFloat(result.Data[0].quantity)
          }))
        }
      }
    } catch (err) {
      console.error('Error fetching balance:', err)
    }
  }

  if (!walletState.isConnected) {
    return (
      <div className="wallet-connect">
        <h1>Connect Your Wallet</h1>
        <button 
          onClick={connectWallet} 
          disabled={walletState.isConnecting}
          className="connect-btn"
        >
          {walletState.isConnecting ? 'Connecting...' : 'Connect MetaMask'}
        </button>
        {walletState.error && (
          <p className="error">{walletState.error}</p>
        )}
      </div>
    )
  }

  return (
    <div className="wallet-app">
      <h1>Wallet Connected</h1>
      <p>Address: {walletState.address}</p>
      <p>Balance: {walletState.balance} GALA</p>
      
      <div className="actions">
        <BurnGala 
          walletAddress={walletState.address}
          metamaskClient={metamaskClient!}
        />
        <TransferGala 
          walletAddress={walletState.address}
          metamaskClient={metamaskClient!}
        />
      </div>
    </div>
  )
}
```

## Summary

This guide provides a comprehensive foundation for integrating MetaMask wallet connection and GalaChain transaction signing into your web application. Based on analysis of production-ready implementations in the GalaChain examples repository, here are the key takeaways:

### Core Implementation Patterns

1. **Service-Based Architecture**: Use a dedicated wallet service to separate wallet logic from UI components
2. **Centralized State Management**: Manage wallet state at the app level and provide it to child components
3. **Backend Integration**: Process transactions through a backend API rather than direct GalaChain calls
4. **Comprehensive Error Handling**: Implement user-friendly error messages and proper error states
5. **Transaction Recording**: Store all transactions in a local database for history and audit trails

### Technical Best Practices

1. **Use `@gala-chain/connect`** for wallet integration with the service pattern
2. **Environment Configuration**: Use descriptive environment variable names (e.g., `VITE_TOKEN_GATEWAY_API`)
3. **Unique Transaction Keys**: Generate unique keys using timestamp + random string
4. **Address Validation**: Support multiple address formats (eth|, client|, 0x)
5. **Loading States**: Provide clear feedback during async operations
6. **Input Validation**: Validate amounts and addresses before processing
7. **Network Error Handling**: Implement retry mechanisms and graceful degradation

### Architecture Benefits

This production pattern provides several advantages:
- **Maintainability**: Clean separation of concerns makes code easier to maintain
- **Scalability**: Backend processing allows for complex business logic
- **Reliability**: Transaction recording ensures data consistency
- **User Experience**: Comprehensive error handling and loading states
- **Security**: Backend validation and processing add security layers

### Recommended Project Structure

```
your-project/
├── frontend/
│   ├── src/
│   │   ├── services/
│   │   │   └── walletService.ts
│   │   ├── components/
│   │   │   ├── WalletConnect.vue
│   │   │   └── TransactionComponent.vue
│   │   └── App.vue
│   └── .env
├── backend/
│   ├── src/
│   │   ├── wallet/
│   │   ├── transaction/
│   │   └── entities/
│   └── package.json
└── README.md
```

The examples in this guide are based on production-ready patterns from the GalaChain examples repository. This implementation demonstrates best practices for secure, user-friendly, and maintainable blockchain applications.
