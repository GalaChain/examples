# MetaMask to GalaChain Connection & Transaction Signing Guide

**Based on the gem-store project implementation**

This guide provides a definitive, step-by-step walkthrough of how to connect MetaMask to GalaChain and use it to sign transactions, based on the working implementation in the gem-store project.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Project Setup](#project-setup)
3. [Dependencies](#dependencies)
4. [Environment Configuration](#environment-configuration)
5. [Wallet Connection Flow](#wallet-connection-flow)
6. [User Registration](#user-registration)
7. [Transaction Signing](#transaction-signing)
8. [Balance Fetching](#balance-fetching)
9. [Complete Code Examples](#complete-code-examples)
10. [Error Handling](#error-handling)
11. [Troubleshooting](#troubleshooting)

---

## Prerequisites

Before you begin, ensure you have:

- **Node.js 18+** installed
- **MetaMask browser extension** installed and set up
- **GALA tokens** in your MetaMask wallet (for testing transactions)
- Basic understanding of:
  - JavaScript/TypeScript
  - Vue.js or React (the patterns work in any framework)
  - Async/await patterns
  - REST API calls

---

## Project Setup

### 1. Install Dependencies

The gem-store project uses the following key dependencies:

```json
{
  "dependencies": {
    "@gala-chain/connect": "^2.3.4",
    "vue": "^3.3.4"
  },
  "devDependencies": {
    "vite": "^4.3.9",
    "vite-plugin-node-polyfills": "^0.9.0"
  }
}
```

**Installation command:**
```bash
npm install @gala-chain/connect
```

### 2. Vite Configuration (for Vite projects)

If you're using Vite, you need to configure polyfills for Node.js modules:

```typescript
// vite.config.ts
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { nodePolyfills } from 'vite-plugin-node-polyfills'

export default defineConfig({
  plugins: [
    vue(),
    nodePolyfills({
      include: ['buffer']
    })
  ]
})
```

**Why?** The `@gala-chain/connect` library uses Node.js modules like `buffer` that need to be polyfilled in the browser.

---

## Dependencies

### Core Library: `@gala-chain/connect`

The `@gala-chain/connect` package provides the `BrowserConnectClient` class, which handles:

- MetaMask wallet connection
- GalaChain address derivation
- Transaction signing
- Public key retrieval

**Key Methods:**
- `connect()` - Connects to MetaMask and derives GalaChain address
- `sign(transactionType, transactionDto)` - Signs a transaction
- `getPublicKey()` - Retrieves the user's public key
- `personalSign(message)` - Signs a message
- `disconnect()` - Disconnects the wallet
- `galaChainAddress` - Property containing the GalaChain-formatted address

---

## Environment Configuration

### Environment Variables

Create a `.env` file in your frontend directory:

```env
# GalaChain API Endpoints
VITE_BURN_GATEWAY_API=https://gateway-mainnet.galachain.com/api/asset/token-contract
VITE_BURN_GATEWAY_PUBLIC_KEY_API=https://gateway-mainnet.galachain.com/api/asset/public-key-contract
VITE_GALASWAP_API=https://api-galaswap.gala.com/galachain

# Project Configuration
VITE_PROJECT_ID=gem-store
VITE_PROJECT_API=http://localhost:4000
VITE_GEM_EXCHANGE_RATE=10
```

### TypeScript Environment Types

Create `env.d.ts` to get TypeScript support for environment variables:

```typescript
/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_BURN_GATEWAY_API: string
  readonly VITE_BURN_GATEWAY_PUBLIC_KEY_API: string
  readonly VITE_GALASWAP_API: string
  readonly VITE_PROJECT_ID: string
  readonly VITE_PROJECT_API: string
  readonly VITE_GEM_EXCHANGE_RATE?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
```

**Note:** For non-Vite projects (React with Webpack, etc.), use `REACT_APP_` prefix instead of `VITE_`.

---

## Wallet Connection Flow

### Step-by-Step Process

The wallet connection process follows these exact steps as implemented in gem-store:

#### 1. Check MetaMask Availability

```typescript
const metamaskSupport = !!window.ethereum
```

**Why?** Not all users have MetaMask installed. Check before attempting connection.

#### 2. Create BrowserConnectClient Instance

```typescript
import { BrowserConnectClient } from '@gala-chain/connect'

const client = new BrowserConnectClient()
```

**Important:** Create a new instance for each connection attempt. Don't reuse instances across sessions.

#### 3. Connect to MetaMask

```typescript
const connected = await client.connect()
```

**What happens:**
- MetaMask popup appears asking user to connect
- User approves or rejects
- If approved, `connect()` returns `true`
- The client automatically derives the GalaChain address

#### 4. Get GalaChain Address

```typescript
if (connected && client.galaChainAddress) {
  const address = client.galaChainAddress
  // Address format: "eth|0x..." or "client|..."
}
```

**Address Format:**
- GalaChain addresses are prefixed with `eth|` or `client|`
- Example: `eth|0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb5`
- This is different from standard Ethereum addresses (which start with `0x`)

#### 5. Complete Connection Flow (Full Example)

```typescript
import { BrowserConnectClient } from '@gala-chain/connect'

async function connectWallet() {
  // Step 1: Check MetaMask availability
  if (!window.ethereum) {
    throw new Error('MetaMask not detected. Please install MetaMask extension.')
  }

  // Step 2: Create client instance
  const client = new BrowserConnectClient()

  try {
    // Step 3: Connect to MetaMask
    const connected = await client.connect()
    
    if (!connected) {
      throw new Error('User rejected connection')
    }

    // Step 4: Get GalaChain address
    if (!client.galaChainAddress) {
      throw new Error('Failed to get GalaChain address')
    }

    const address = client.galaChainAddress
    
    // Step 5: Store client for later use (signing transactions)
    // Store client in a service or state management
    
    return {
      address,
      client // Keep client for signing transactions
    }
  } catch (error) {
    console.error('Connection error:', error)
    throw error
  }
}
```

---

## User Registration

### Why Registration is Needed

GalaChain requires users to be registered before they can interact with the blockchain. The registration process:

1. Checks if user is already registered
2. If not registered, automatically registers the user
3. Associates the user's public key with their GalaChain address

### Registration Check

```typescript
async function checkRegistration(address: string): Promise<boolean> {
  const response = await fetch(
    `${import.meta.env.VITE_BURN_GATEWAY_PUBLIC_KEY_API}/GetPublicKey`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ user: address })
    }
  )
  
  if (!response.ok) {
    throw new Error('User not registered')
  }
  
  return true
}
```

**What this does:**
- Queries GalaChain's public key gateway API
- If user exists, returns their public key
- If user doesn't exist, returns an error

### Auto-Registration

```typescript
async function registerUser(client: BrowserConnectClient): Promise<void> {
  // Step 1: Get public key from MetaMask
  const publicKeyResult = await client.getPublicKey()
  
  // Step 2: Register with GalaChain
  const response = await fetch(
    `${import.meta.env.VITE_GALASWAP_API}/CreateHeadlessWallet`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ 
        publicKey: publicKeyResult.publicKey 
      })
    }
  )
  
  if (!response.ok) {
    throw new Error('Failed to register user')
  }
}
```

**What this does:**
- Retrieves the user's public key from MetaMask
- Sends the public key to GalaChain's registration API
- Creates a "headless wallet" (wallet without a UI) on GalaChain

### Complete Registration Flow

```typescript
async function connectAndRegister() {
  const client = new BrowserConnectClient()
  await client.connect()
  
  const address = client.galaChainAddress
  
  // Check registration
  try {
    await checkRegistration(address)
    console.log('User already registered')
  } catch (e) {
    console.log('User not registered, registering...')
    await registerUser(client)
    console.log('User registered successfully')
  }
  
  return { address, client }
}
```

---

## Transaction Signing

### Understanding GalaChain Transactions

GalaChain transactions follow a specific structure:

1. **Transaction DTO (Data Transfer Object)** - The transaction data
2. **Signing** - MetaMask signs the transaction
3. **Submission** - Submit signed transaction to GalaChain API

### Example: BurnTokens Transaction

The gem-store project uses `BurnTokens` as an example. Here's the complete flow:

#### 1. Create Transaction DTO

```typescript
const burnTokensDto = {
  owner: walletAddress, // GalaChain address (eth|0x...)
  tokenInstances: [{
    quantity: galaAmount.toString(), // Amount to burn
    tokenInstanceKey: {
      collection: "GALA",
      category: "Unit",
      type: "none",
      additionalKey: "none",
      instance: "0"
    }
  }],
  uniqueKey: `gem-purchase-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
}
```

**Key Fields Explained:**
- `owner`: The GalaChain address of the transaction sender
- `tokenInstances`: Array of tokens to burn
  - `quantity`: Amount as a string (GalaChain uses string numbers)
  - `tokenInstanceKey`: Identifies the token type
    - `collection`: "GALA" for GALA tokens
    - `category`: "Unit" for fungible tokens
    - `type`, `additionalKey`: Usually "none"
    - `instance`: "0" for fungible tokens
- `uniqueKey`: Unique identifier for this transaction (prevents duplicates)

#### 2. Sign Transaction with MetaMask

```typescript
if (!client) {
  throw new Error('Wallet not connected')
}

const signedTransaction = await client.sign("BurnTokens", burnTokensDto)
```

**What happens:**
- MetaMask popup appears showing transaction details
- User reviews and approves/rejects
- If approved, transaction is cryptographically signed
- Returns signed transaction object ready for submission

**Important:** The `client` must be the same instance used during connection.

#### 3. Submit to GalaChain

```typescript
const response = await fetch(
  `${import.meta.env.VITE_BURN_GATEWAY_API}/BurnTokens`,
  {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(signedTransaction)
  }
)

if (!response.ok) {
  throw new Error(`Transaction failed: ${response.status}`)
}

const result = await response.json()
const transactionId = result.transactionId
```

**What happens:**
- Signed transaction is sent to GalaChain gateway API
- GalaChain validates the signature
- Transaction is processed on the blockchain
- Returns transaction ID for tracking

### Complete Transaction Flow Example

```typescript
async function burnTokens(
  client: BrowserConnectClient,
  walletAddress: string,
  galaAmount: number
) {
  // Step 1: Create transaction DTO
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
    uniqueKey: `burn-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
  }

  // Step 2: Sign with MetaMask
  const signedTransaction = await client.sign("BurnTokens", burnTokensDto)

  // Step 3: Submit to GalaChain
  const response = await fetch(
    `${import.meta.env.VITE_BURN_GATEWAY_API}/BurnTokens`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(signedTransaction)
    }
  )

  if (!response.ok) {
    const error = await response.json()
    throw new Error(error.message || 'Transaction failed')
  }

  const result = await response.json()
  return {
    success: true,
    transactionId: result.transactionId
  }
}
```

---

## Balance Fetching

### Fetching GALA Balance

GalaChain uses a specific API endpoint to fetch token balances:

```typescript
async function fetchGalaBalance(walletAddress: string): Promise<number> {
  const balanceDto = {
    owner: walletAddress,
    collection: "GALA",
    category: "Unit",
    type: "none",
    additionalKey: "none",
    instance: "0"
  }

  const response = await fetch(
    `${import.meta.env.VITE_BURN_GATEWAY_API}/FetchBalances`,
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
  
  // Response structure: { Data: [{ quantity: "100.0", ... }] }
  if (result.Data && result.Data.length > 0) {
    return parseFloat(result.Data[0].quantity)
  }
  
  return 0
}
```

**Response Structure:**
```json
{
  "Data": [
    {
      "quantity": "100.5",
      "tokenInstanceKey": {
        "collection": "GALA",
        "category": "Unit",
        "type": "none",
        "additionalKey": "none",
        "instance": "0"
      }
    }
  ]
}
```

**Important Notes:**
- `quantity` is returned as a string
- Use `parseFloat()` to convert to number
- If user has no balance, `Data` array may be empty

---

## Complete Code Examples

### Wallet Service (Complete Implementation)

Based on gem-store's `walletService.ts`:

```typescript
import { BrowserConnectClient } from '@gala-chain/connect'

export interface ConnectionResult {
  address: string
  provider: BrowserConnectClient
}

export const walletService = {
  client: null as BrowserConnectClient | null,
  
  async connect(): Promise<ConnectionResult> {
    // Check MetaMask availability
    if (!window.ethereum) {
      throw new Error('MetaMask not detected. Please install MetaMask extension.')
    }

    // Create new client instance
    this.client = new BrowserConnectClient()
    
    // Connect to MetaMask
    const connected = await this.client.connect()
    
    if (!connected || !this.client.galaChainAddress) {
      throw new Error('Failed to connect wallet')
    }
    
    return {
      address: this.client.galaChainAddress,
      provider: this.client
    }
  },
  
  async signTransaction(transactionType: string, transactionDto: any) {
    if (!this.client) {
      throw new Error('Wallet not connected')
    }
    
    return await this.client.sign(transactionType, transactionDto)
  },
  
  async getPublicKey() {
    if (!this.client) {
      throw new Error('Wallet not connected')
    }
    
    return await this.client.getPublicKey()
  },
  
  async signMessage(message: string) {
    if (!this.client) {
      throw new Error('Wallet not connected')
    }
    
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

### Wallet Connect Component (Vue Example)

Based on gem-store's `WalletConnect.vue`:

```vue
<template>
  <div>
    <button 
      @click="connectWallet" 
      :disabled="isConnecting"
    >
      {{ isConnecting ? 'Connecting...' : 'Connect MetaMask' }}
    </button>
    
    <div v-if="error" class="error">
      {{ error }}
    </div>
    
    <div v-if="isConnected">
      <p>Address: {{ walletAddress }}</p>
      <p>Balance: {{ galaBalance }} GALA</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { BrowserConnectClient } from '@gala-chain/connect'
import { walletService } from '../services/walletService'

const isConnected = ref(false)
const isConnecting = ref(false)
const walletAddress = ref('')
const galaBalance = ref(0)
const error = ref('')

const connectWallet = async () => {
  try {
    isConnecting.value = true
    error.value = ''
    
    // Connect wallet
    const connectionResult = await walletService.connect()
    walletAddress.value = connectionResult.address
    
    // Check registration (optional - skip if user is already registered)
    try {
      await checkRegistration(walletAddress.value)
    } catch (e) {
      console.log('User not registered, registering...')
      await registerUser()
    }
    
    isConnected.value = true
    
    // Fetch balance
    await fetchBalance(walletAddress.value)
    
  } catch (err) {
    console.error('Error connecting wallet:', err)
    error.value = err instanceof Error ? err.message : 'Failed to connect wallet'
  } finally {
    isConnecting.value = false
  }
}

const checkRegistration = async (address: string) => {
  const response = await fetch(
    `${import.meta.env.VITE_BURN_GATEWAY_PUBLIC_KEY_API}/GetPublicKey`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ user: address })
    }
  )
  if (!response.ok) throw new Error('User not registered')
}

const registerUser = async () => {
  const publicKey = await walletService.getPublicKey()
  const response = await fetch(
    `${import.meta.env.VITE_GALASWAP_API}/CreateHeadlessWallet`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ publicKey: publicKey.publicKey })
    }
  )
  if (!response.ok) throw new Error('Failed to register user')
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
      `${import.meta.env.VITE_BURN_GATEWAY_API}/FetchBalances`,
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
</script>
```

### Transaction Signing Component (Vue Example)

Based on gem-store's `GemStore.vue`:

```vue
<template>
  <div>
    <div v-if="!walletAddress">
      Please connect your wallet first.
    </div>
    
    <div v-else>
      <p>GALA Balance: {{ galaBalance }} GALA</p>
      
      <button 
        @click="purchaseGems" 
        :disabled="isProcessing || !canPurchase"
      >
        {{ isProcessing ? 'Processing...' : 'Burn 1 GALA for 10 Gems' }}
      </button>
      
      <div v-if="status" :class="statusType">
        {{ status }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { walletService } from '../services/walletService'

const props = defineProps<{
  walletAddress: string
}>()

const galaBalance = ref(0)
const isProcessing = ref(false)
const status = ref('')
const statusType = ref('')

const canPurchase = computed(() => {
  return galaBalance.value >= 1
})

const purchaseGems = async () => {
  if (!props.walletAddress) return
  
  try {
    isProcessing.value = true
    status.value = ''
    
    // Step 1: Create burn transaction DTO
    const burnTokensDto = {
      owner: props.walletAddress,
      tokenInstances: [{
        quantity: "1", // Amount to burn
        tokenInstanceKey: {
          collection: "GALA",
          category: "Unit",
          type: "none",
          additionalKey: "none",
          instance: "0"
        }
      }],
      uniqueKey: `gem-purchase-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
    }

    // Step 2: Sign the transaction with MetaMask
    const signedTransaction = await walletService.signTransaction("BurnTokens", burnTokensDto)
    
    // Step 3: Submit to backend (or directly to GalaChain)
    const response = await fetch('/api/transactions/burn', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        signedTransaction: signedTransaction,
        gemAmount: 10,
        galaAmount: 1,
        walletAddress: props.walletAddress
      })
    })

    if (!response.ok) {
      throw new Error('Failed to process transaction')
    }

    const result = await response.json()
    
    // Update balance
    galaBalance.value -= 1
    
    // Show success
    status.value = `Successfully purchased 10 gems! Transaction: ${result.transactionId}`
    statusType.value = 'success'
    
  } catch (error) {
    console.error('Error purchasing gems:', error)
    status.value = `Error: ${error instanceof Error ? error.message : 'Failed to purchase gems'}`
    statusType.value = 'error'
  } finally {
    isProcessing.value = false
  }
}
</script>
```

---

## Error Handling

### Common Error Scenarios

#### 1. MetaMask Not Installed

```typescript
if (!window.ethereum) {
  throw new Error('MetaMask not detected. Please install MetaMask extension.')
}
```

**User-friendly message:**
```typescript
error.value = 'MetaMask not detected. Please install MetaMask from https://metamask.io/'
```

#### 2. User Rejects Connection

```typescript
try {
  const connected = await client.connect()
  if (!connected) {
    throw new Error('User rejected connection')
  }
} catch (error) {
  if (error.code === 4001) {
    // User rejected the request
    error.value = 'Connection rejected. Please try again.'
  }
}
```

#### 3. User Rejects Transaction Signing

```typescript
try {
  const signedTransaction = await client.sign("BurnTokens", burnTokensDto)
} catch (error) {
  if (error.code === 4001) {
    error.value = 'Transaction signing rejected. Please try again.'
  } else {
    error.value = 'Transaction signing failed. Please try again.'
  }
}
```

#### 4. Network Errors

```typescript
try {
  const response = await fetch(apiUrl, options)
  if (!response.ok) {
    throw new Error(`API error: ${response.status}`)
  }
} catch (error) {
  if (error instanceof TypeError && error.message.includes('fetch')) {
    error.value = 'Network error. Please check your connection.'
  } else {
    error.value = error.message
  }
}
```

#### 5. Insufficient Balance

```typescript
const canPurchase = computed(() => {
  return galaBalance.value >= requiredAmount
})

if (!canPurchase.value) {
  throw new Error(`Insufficient balance. You need ${requiredAmount} GALA but have ${galaBalance.value} GALA.`)
}
```

### Complete Error Handling Example

```typescript
async function connectWalletWithErrorHandling() {
  try {
    // Check MetaMask
    if (!window.ethereum) {
      throw new Error('METAMASK_NOT_INSTALLED')
    }

    // Connect
    const client = new BrowserConnectClient()
    const connected = await client.connect()
    
    if (!connected) {
      throw new Error('CONNECTION_REJECTED')
    }

    if (!client.galaChainAddress) {
      throw new Error('ADDRESS_NOT_FOUND')
    }

    return { success: true, address: client.galaChainAddress, client }
    
  } catch (error) {
    // Handle specific error codes
    if (error.code === 4001) {
      return { success: false, error: 'CONNECTION_REJECTED' }
    }
    
    if (error.message === 'METAMASK_NOT_INSTALLED') {
      return { success: false, error: 'METAMASK_NOT_INSTALLED' }
    }
    
    if (error.message === 'CONNECTION_REJECTED') {
      return { success: false, error: 'CONNECTION_REJECTED' }
    }
    
    // Generic error
    return { success: false, error: 'UNKNOWN_ERROR', details: error.message }
  }
}
```

---

## Troubleshooting

### Issue: "MetaMask not detected"

**Solutions:**
1. Ensure MetaMask extension is installed
2. Refresh the page after installing MetaMask
3. Check if MetaMask is enabled in browser extensions
4. Try a different browser

### Issue: "Failed to connect wallet"

**Solutions:**
1. Check browser console for detailed errors
2. Ensure MetaMask is unlocked
3. Try disconnecting and reconnecting
4. Clear browser cache and cookies

### Issue: "User not registered" error persists

**Solutions:**
1. Check network connection
2. Verify API endpoints in `.env` file
3. Check browser console for API errors
4. Try manual registration:
   ```typescript
   const publicKey = await client.getPublicKey()
   // Manually call CreateHeadlessWallet API
   ```

### Issue: Transaction signing fails

**Solutions:**
1. Ensure wallet is connected
2. Check that `client` instance is the same from connection
3. Verify transaction DTO structure matches GalaChain requirements
4. Check MetaMask is unlocked
5. Verify user has sufficient balance

### Issue: "Buffer is not defined" error

**Solutions:**
1. Install `vite-plugin-node-polyfills`
2. Configure Vite to include buffer polyfill:
   ```typescript
   nodePolyfills({
     include: ['buffer']
   })
   ```

### Issue: Balance shows 0 but user has GALA

**Solutions:**
1. Verify wallet address format (should be `eth|0x...`)
2. Check API endpoint is correct
3. Verify balance DTO structure
4. Check network (mainnet vs testnet)

### Issue: Transaction submission fails

**Solutions:**
1. Verify signed transaction structure
2. Check API endpoint URL
3. Verify transaction was actually signed (check signedTransaction object)
4. Check GalaChain API status
5. Verify uniqueKey is unique

---

## Best Practices

### 1. Store Client Instance

```typescript
// ✅ Good: Store client for reuse
const client = new BrowserConnectClient()
await client.connect()
// Store client in service/state

// ❌ Bad: Creating new client for each operation
await new BrowserConnectClient().connect()
await new BrowserConnectClient().sign(...) // Won't work!
```

### 2. Handle User Rejections Gracefully

```typescript
// ✅ Good: Handle rejection
try {
  await client.connect()
} catch (error) {
  if (error.code === 4001) {
    // User rejected - don't show error, just return
    return
  }
  throw error
}
```

### 3. Validate Before Signing

```typescript
// ✅ Good: Validate before signing
if (galaBalance < requiredAmount) {
  throw new Error('Insufficient balance')
}
await client.sign(...)

// ❌ Bad: Sign first, validate later
await client.sign(...) // User pays gas for failed transaction
```

### 4. Use Unique Keys

```typescript
// ✅ Good: Unique key prevents duplicate transactions
uniqueKey: `transaction-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`

// ❌ Bad: Reusable key can cause issues
uniqueKey: 'my-transaction'
```

### 5. Error Messages for Users

```typescript
// ✅ Good: User-friendly messages
error.value = 'Insufficient balance. You need 10 GALA but have 5 GALA.'

// ❌ Bad: Technical error messages
error.value = 'Error: InsufficientFundsException: balance 5 < required 10'
```

---

## API Reference Summary

### GalaChain API Endpoints

#### 1. Get Public Key (Registration Check)
```
POST https://gateway-mainnet.galachain.com/api/asset/public-key-contract/GetPublicKey
Body: { user: "eth|0x..." }
```

#### 2. Create Headless Wallet (Registration)
```
POST https://api-galaswap.gala.com/galachain/CreateHeadlessWallet
Body: { publicKey: "..." }
```

#### 3. Fetch Balances
```
POST https://gateway-mainnet.galachain.com/api/asset/token-contract/FetchBalances
Body: {
  owner: "eth|0x...",
  collection: "GALA",
  category: "Unit",
  type: "none",
  additionalKey: "none",
  instance: "0"
}
```

#### 4. Burn Tokens
```
POST https://gateway-mainnet.galachain.com/api/asset/token-contract/BurnTokens
Body: <signed transaction from client.sign()>
```

---

## Testing Checklist

Before deploying, test:

- [ ] MetaMask connection works
- [ ] GalaChain address is correctly derived
- [ ] User registration check works
- [ ] Auto-registration works for new users
- [ ] Balance fetching works
- [ ] Transaction signing works
- [ ] Transaction submission works
- [ ] Error handling for rejected connections
- [ ] Error handling for rejected transactions
- [ ] Error handling for insufficient balance
- [ ] Error handling for network errors
- [ ] Disconnect functionality works
- [ ] Reconnection after disconnect works

---

## Additional Resources

- [GalaChain Connect Library Documentation](https://github.com/GalaGames/gala-chain-connect)
- [MetaMask Documentation](https://docs.metamask.io/)
- [GalaChain Gateway API](https://gateway-mainnet.galachain.com)

---

## Conclusion

This guide provides a complete, production-ready implementation pattern for connecting MetaMask to GalaChain and signing transactions, based on the working gem-store project. Follow these patterns exactly, and you'll have a robust wallet integration.

**Key Takeaways:**
1. Always check MetaMask availability before connecting
2. Store the `BrowserConnectClient` instance for transaction signing
3. Handle user rejections gracefully
4. Validate balances before signing transactions
5. Use unique keys for all transactions
6. Provide clear, user-friendly error messages

For questions or issues, refer to the gem-store project source code as the definitive reference implementation.

