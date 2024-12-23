# GalaChain dApp Quick Start Guide

## Overview
This guide will help you build a simple dApp that interacts with GalaChain mainnet using the GalaChain Connect library. We'll cover user registration, balance checks, transaction signing, and more.

## Prerequisites
- Node.js environment
- Metamask wallet
- Basic understanding of TypeScript/JavaScript
- GalaChain Connect library
- axios or other HTTP client library

## Installation

```bash
npm install @gala-games/chain-connect
```

## Wallet Connection

Allow the user to connect their Metamask wallet to the dApp.

```typescript  

    import { MetamaskConnectClient } from '@gala-chain/connect'

    const metamaskClient = new MetamaskConnectClient();

    let walletAddress = ''

    try {
        await metamaskClient.connect()

        let address = metamaskClient.getWalletAddress

        if (address.startsWith('0x')) {
            walletAddress = "eth|" + address.slice(2)
        }
    } catch (err) {
        console.error('Error connecting wallet:', err)
    }
```

## Check Registration

```typescript
  try {
    const response = await axios.post(`${PUBLIC_KEY_API_BASE_URL}/GetPublicKey`, {
      user: walletAddress
    })
  } catch (err) {
    // not really an error, just means the user is not registered yet
    console.log('User is not registered', err)
  }
```

## User Registration

```typescript
  try {
    const publicKey = await metamaskClient.getPublicKey()

    const registerDto = {
      publicKey: publicKey.publicKey
    }

    const response = await axios.post(`${GALASWAP_API_BASE_URL}/CreateHeadlessWallet`, registerDto)
  } catch (err) {
    console.error('Error registering user:', err)
  }
```

## Checking Token Balances

Once authenticated, you can check token balances:

``` typescript
  try {
    const balanceDto = {
      owner: walletAddress, // user's registered GalaChain address e.g. eth|1234567890...
      collection: "GALA", // GALA token used here as an example but any token key can be used
      category: "Unit",
      type: "none",
      additionalKey: "none",
      instance: "0"
    }

    const response = await axios.post(`${TOKEN_API_BASE_URL}/FetchBalances`, balanceDto);

    let balance = 0

    if (response.data.Data.length > 0) {
      balance = parseFloat(response.data.Data[0].quantity)
      
      // you can extract the locked amount of the balance here and note or subtract it from the balance
      let lockedBalance = response.data.Data[0].lockedHolds.reduce(
        (acc: number, hold: any) => acc + parseFloat(hold.quantity), 0);
    }
  } catch (err) {
    console.error(`Error fetching balance:`, err)
  }
);
```

## Executing Transactions

### Token Burns

Here's how to implement a token burn transaction:

``` typescript
  try {
    const owner: walletAddress, // user's registered GalaChain address e.g. eth|1234567890...
    const burnAmount = 10 // some kind of input from the user or dApp

    const burnTokensDto = { 
      owner,
      tokenInstances: [{
        quantity: burnAmount,
        tokenInstanceKey: {
            collection: "GALA",
            category: "Unit",
            type: "none",
            additionalKey: "none",
            instance: "0"
        }
      }],
      uniqueKey: `my-dapp-burn-${Date.now()}` // all write transactions must have a unique key, can be useful for tracking
    }

	const signedBurnDto = metamaskClient.sign("BurnTokens", burnTokensDto)
	const response = await axios.post(`${TOKEN_API_BASE_URL}/BurnTokens`, signedBurnDto)
  } catch (error) {
    console.error('Error burning GALA:', error)
  }
```

## API Endpoints Reference

### Docs
- Authentication: `https://gateway.mainnet.galachain.com/docs/?channel=asset&contract=public-key-contract`
- Token Operations: `https://gateway.mainnet.galachain.com/docs/?channel=asset&contract=token-contract`
- Blocks/Transactions: `https://explorer-api.galachain.com/docs`
- GalaSwap API: `https://connect.gala.com/info/api.html`

### Common API Patterns
- POST `/api/asset/token-contract/BurnTokens`
- POST `/api/asset/token-contract/FetchBalances`
- POST `/api/asset/public-key-contract/GetPublicKey`
- POST `https://api-galaswap.gala.com/galachain/CreateHeadlessWallet`

## Block Explorer

GalaChain provides a block explorer for monitoring transactions:
- Mainnet: `https://explorer.galachain.com`

Block Explorer Features:
- Transaction history
- Block information
- Chaincode interactions

## Additional Resources

- [SDK](https://github.com/GalaChain/sdk)
- [GalaChain Documentation](https://docs.galachain.com)
- [Developer Discord](https://discord.gg/galachain)