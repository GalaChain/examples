# Burn to Vote - Sample GalaChain Burn dApp

Burn $GALA to upvote.

A lightweight application that allows users to connect their wallet, check their GALA balance, and burn GALA tokens on mainnet.

## Prerequisites

- Node.js (v14 or later)
- npm or yarn
- MetaMask wallet

## Setup and Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/GalaChain/examples.git
   cd examples/dapp-template
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Create a `.env` file in the root directory with the following content:
   ```
   VITE_BURN_GATEWAY_API=https://gateway-mainnet.galachain.com/api/asset/token-contract
   VITE_BURN_GATEWAY_PUBLIC_KEY_API=https://gateway-mainnet.galachain.com/api/asset/public-key-contract
   VITE_GALASWAP_API=https://api-galaswap.gala.com/galachain
   VITE_PROJECT_ID=<my project id>
   VITE_PROJECT_API=http://localhost:4000
   VITE_BURN_COST_SUBMIT=10
   VITE_BURN_COST_VOTE=1
   ```

By default, `trailbase` runs on port 4000. If you configure it to run on another port, or are using a remote URL, edit the `VITE_PROJECT_API` property above accordingly.

Create a separate `trailbase.env.js` file for configuration of the `trailbase` backend server's JavaScript / scripts directory with the following content:

```
export const BURN_GATEWAY_API="https://gateway-mainnet.galachain.com/api/asset/token-contract"
export const BURN_GATEWAY_PUBLIC_KEY_API="https://gateway-mainnet.galachain.com/api/asset/public-key-contract";
export const PROJECT_ID="readwriteburn";
export const BURN_COST_SUBMIT=10;
export const BURN_COST_VOTE=1;
```

Trailbase supports TypeScript based scripting for custom API routes, which we use to make some HTTP / Fetch calls to the GalaChain Mainnet from our basic backend server.

4. Start the trailbase backend.

Using Docker:

```bash
alias trail="docker run -p 4000:4000 --mount type=bind,source=$PWD/traildepot,target=/app/traildepot trailbase/trailbase /app/trail"
trail run --dev
```

Or [download a pre-built binary](https://github.com/trailbaseio/trailbase/releases/) for your architecture locally to the project root and run

```
./trail run --dev
```

The `--dev` flag is useful for local development, to support CORS (Cross-Origin Resource Sharing) from your front end client app running on port 3001 (see below) to the trailbase backend server running on port 4000.

When trailbase start for the first time, it will create a new sqlite database, run the schema migrations, and create an initial administrative user.

Look for the username/password of the new admin user in the log output - you will need these details to login to the admin UI.

Open [http://127.0.0.1:4000/_/admin](http://127.0.0.1:4000/_/admin) in a web browser to access the trailbase administrative user interface. Login with the credentials created on inital startup.

Click the pen and paper icon in the left hand side bar to access the [SQL Editor](http://127.0.0.1:4000/_/admin/editor). Review each of the .sql files in resources and paste them into the editor to execute SQL to populate initial tables.

If you try to add these files directly to the database using `sqlite3 traildepot/data/main.db < resources/{data insert file}.sql`, you may receive an error `unknown function: is_uuid_v7()`. This is an application-defined sqlite function used by trailbase that is not available in a standard, compiled version. Use the editor in the admin UI in order to run the SQL statements through trailbase's application layer and you won't have this problem.

4. Next start the front-end development server:

   ```bash
   npm run dev
   ```

Load the front-end application at [http://localhost:3001](http://localhost:3001).

## Features

- MetaMask wallet connection
- Automatic user registration with GalaChain if needed
- GALA token balance display (including locked amounts)
- Token burning functionality
- Off-chain backend API and sqlite persistence layer using [Trailbase](https://trailbase.io/)
- Administrative UI, point and click to create new database tables, add indices, delete rows, etc.

## Project Structure

- `src/App.vue` - Main application component
- `src/components/`
  - `Balance.vue` - Displays GALA balance
  - `BurnGala.vue` - Handles token burning
  - `WalletConnect.vue` - Handles wallet connection
- Frontend Environment variables are defined in `.env`
- Vite configuration in `vite.config.ts`
- Backend Environment variables are defined in `traildepot/trailbase.env.js`
- Typescript code for custom backend routes are defined in `traildepot/scripts/main.ts`

## Usage

1. Open your browser and navigate to `http://localhost:3001`
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
- [Trailbase](https://trailbase.io/)
- GalaChain Connect library

## Additional Resources

- [GalaChain Documentation](https://docs.galachain.com)
- [GalaChain Connect Library](https://github.com/GalaChain/sdk)
