# GalaChain MetaMask Wallet Connection Guide

This guide provides a comprehensive overview of how to integrate MetaMask wallet connection and GalaChain transaction signing into your web application. Based on analysis of the GalaChain examples codebase, this document covers best practices, code patterns, and implementation details.

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
npm install @gala-chain/connect
```

For React applications, you may also need:

```bash
npm install @gala-chain/api
```

## Environment Configuration

Create a `.env` file in your project root with the following variables:

```env
# GalaChain API Endpoints
TOKEN_GATEWAY_API=https://gateway-mainnet.galachain.com/api/asset/token-contract
PUBLIC_KEY_GATEWAY_API=https://gateway-mainnet.galachain.com/api/asset/public-key-contract
CONNECT_API=https://api-galaswap.gala.com/galachain

# Project Configuration
VITE_PROJECT_ID=your-project-id
VITE_PROJECT_API=http://localhost:4000
```

Define TypeScript interfaces for environment variables:

```typescript
// env.d.ts
interface ImportMetaEnv {
  readonly TOKEN_GATEWAY_API: string
  readonly PUBLIC_KEY_GATEWAY_API: string
  readonly CONNECT_API: string
  readonly VITE_PROJECT_ID: string
  readonly VITE_PROJECT_API: string
}
```

## Wallet Connection

### Basic Connection Setup

```typescript
import { BrowserConnectClient } from '@gala-chain/connect'

// Initialize the client
const metamaskClient = new BrowserConnectClient()

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
    
    // Connect to MetaMask
    await metamaskClient.connect()
    
    // Get GalaChain address
    const address = metamaskClient.galaChainAddress
    setWalletAddress(address)
    setIsConnected(true)
    
    // Check if user is registered
    await checkRegistration(address)
    
  } catch (error) {
    console.error('Error connecting wallet:', error)
    throw error
  } finally {
    setIsConnecting(false)
  }
}
```

### Vue.js Implementation

```vue
<template>
  <div class="wallet-connect">
    <button @click="connectWallet" v-if="!isConnected" class="button">
      Connect Wallet
    </button>
    <div v-else>
      <p class="wallet-address">Connected: {{ walletAddress }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { BrowserConnectClient } from '@gala-chain/connect'

const metamaskClient = new BrowserConnectClient()
const isConnected = ref(false)
const walletAddress = ref('')

const connectWallet = async () => {
  try {
    await metamaskClient.connect()
    walletAddress.value = metamaskClient.galaChainAddress
    isConnected.value = true
    await checkRegistration()
  } catch (err) {
    console.error('Error connecting wallet:', err)
  }
}
</script>
```

### React Implementation

```tsx
import { useState, useEffect } from 'react'
import { BrowserConnectClient } from '@gala-chain/connect'

export default function WalletConnect({ onAddressChange }: { onAddressChange: (address: string) => void }) {
  const [account, setAccount] = useState<string>('')
  const [isConnecting, setIsConnecting] = useState(false)
  const [metamaskClient, setMetamaskClient] = useState<BrowserConnectClient | null>(null)

  useEffect(() => {
    setMetamaskClient(new BrowserConnectClient())
  }, [])

  const connectWallet = async () => {
    if (!metamaskClient) return

    try {
      setIsConnecting(true)
      await metamaskClient.connect()
      const address = metamaskClient.galaChainAddress
      setAccount(address)
      onAddressChange(address)
      await checkRegistration(address)
    } catch (error) {
      console.error('Failed to connect wallet:', error)
    } finally {
      setIsConnecting(false)
    }
  }

  return (
    <div>
      {account ? (
        <div>
          <p>Connected Account: {account}</p>
          <button onClick={() => metamaskClient?.disconnect()}>
            Disconnect
          </button>
        </div>
      ) : (
        <button onClick={connectWallet} disabled={isConnecting}>
          {isConnecting ? 'Connecting...' : 'Connect Wallet'}
        </button>
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
      `${import.meta.env.PUBLIC_KEY_GATEWAY_API}/GetPublicKey`,
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

### Register New User

```typescript
async function registerUser(metamaskClient: BrowserConnectClient) {
  try {
    // Get public key from MetaMask
    const publicKey = await metamaskClient.getPublicKey()
    
    // Register with GalaChain
    const response = await fetch(
      `${import.meta.env.CONNECT_API}/CreateHeadlessWallet`,
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
    await metamaskClient.connect()
    const address = metamaskClient.galaChainAddress
    
    // Check registration
    try {
      await checkRegistration(address)
    } catch (e) {
      console.log(`Registration check failed: ${e}. Attempting to register user: ${address}`)
      await registerUser(metamaskClient)
    }
    
    setWalletAddress(address)
    setIsConnected(true)
  } catch (err) {
    console.error('Error connecting wallet:', err)
  }
}
```

## Transaction Signing

### Token Burn Transaction

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
      uniqueKey: `burn-${Date.now()}`
    }

    // Sign the transaction
    const signature = await metamaskClient.sign(burnTokensDto)

    // Submit to GalaChain
    const response = await fetch(
      `${import.meta.env.TOKEN_GATEWAY_API}/BurnTokens`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(signature)
      }
    )

    if (!response.ok) {
      throw new Error('Failed to burn tokens')
    }

    const result = await response.json()
    return result
  } catch (error) {
    console.error('Error burning tokens:', error)
    throw error
  }
}
```

### Token Transfer Transaction

```typescript
async function transferTokens(
  from: string,
  to: string,
  amount: number,
  metamaskClient: BrowserConnectClient
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
      uniqueKey: `transfer-${Date.now()}`
    }

    // Sign the transaction
    const signedTransferDto = await metamaskClient.sign("TransferTokens", transferTokensDto)

    // Submit to GalaChain
    const response = await fetch(
      `${import.meta.env.TOKEN_GATEWAY_API}/TransferToken`,
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
      `${import.meta.env.TOKEN_GATEWAY_API}/FetchBalances`,
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
const galaAddress = formatGalaChainAddress(metamaskClient.getWalletAddress)
```

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
  return import.meta.env.TOKEN_GATEWAY_API
}
```

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

This guide provides a comprehensive foundation for integrating MetaMask wallet connection and GalaChain transaction signing into your web application. Key takeaways:

1. **Use `@gala-chain/connect`** for wallet integration
2. **Always check user registration** before allowing transactions
3. **Implement proper error handling** for all wallet operations
4. **Use unique transaction keys** to prevent replay attacks
5. **Provide clear loading states** and user feedback
6. **Validate inputs** before submitting transactions
7. **Handle network errors** gracefully

The examples in this guide are based on real implementations from the GalaChain examples repository and follow established patterns for secure, user-friendly wallet integration.
