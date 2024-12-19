# GalaChain Burn dApp

A lightweight application that allows users to connect their wallet, check their GALA balance, and burn GALA tokens on mainnet.

## Prerequisites

- Node.js (v14 or later)
- npm or yarn
- MetaMask wallet

## Setup and Installation

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd galachain-burn-dapp
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Create a `.env` file in the root directory with the following content:
   ```
   VITE_BURN_GATEWAY_API=https://gateway.mainnet.galachain.com/api/asset/token-contract
   VITE_BURN_GATEWAY_PUBLIC_KEY_API=https://gateway-mainnet.galachain.com/api/asset/public-key-contract
   VITE_GALASWAP_API=https://api-galaswap.gala.com/v1
   VITE_PROJECT_ID=<my project id>
   ```

4. Start the development server:
   ```bash
   npm run dev
   ```

## Features

- MetaMask wallet connection
- Automatic user registration with GalaChain if needed
- GALA token balance display (including locked amounts)
- Token burning functionality

## Project Structure

- `src/App.vue` - Main application component
- `src/components/`
  - `Balance.vue` - Displays GALA balance
  - `BurnGala.vue` - Handles token burning
  - `WalletConnect.vue` - Handles wallet connection
- Environment variables are defined in `.env`
- Vite configuration in `vite.config.ts`

## Usage

1. Open your browser and navigate to `http://localhost:3000`
2. Click "Connect Wallet" to connect your MetaMask wallet
3. Once connected, you'll see your GALA balance
4. Enter the amount of GALA you want to burn
5. Click "Burn Tokens" to initiate the transaction
6. Confirm the transaction in MetaMask

## Development

The application is built with:
- Vue 3 (Composition API)
- TypeScript
- Vite
- GalaChain Connect library

## Additional Resources

- [GalaChain Documentation](https://docs.galachain.com)
- [GalaChain Connect Library](https://github.com/GalaChain/sdk)

