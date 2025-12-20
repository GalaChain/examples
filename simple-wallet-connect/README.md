# Simple GalaChain Wallet Connection

A minimal example of connecting a MetaMask wallet to GalaChain using plain HTML and JavaScript (no Vue, React, or other frameworks).

## Setup

1. Install dependencies:
```bash
npm install
```

2. Start the development server:
```bash
npm run dev
```

3. Open your browser to the URL shown (usually `http://localhost:5173`)

4. Make sure you have MetaMask installed in your browser

5. Click "Connect Wallet" to connect your MetaMask wallet

## How It Works

- **index.html**: Simple HTML page with a connect button
- **wallet.js**: Plain JavaScript that uses `@gala-chain/connect` to connect to MetaMask
- Uses ES modules (no build step required for development)
- Uses Vite as a simple dev server to handle module imports

## Key Code

The wallet connection is handled in `wallet.js`:

```javascript
import { BrowserConnectClient } from '@gala-chain/connect';

const client = new BrowserConnectClient();
const connected = await client.connect();
const address = client.galaChainAddress;
```

That's it! No framework needed.

