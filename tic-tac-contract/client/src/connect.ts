import { Ref } from "vue";
import { BrowserConnectClient } from "@gala-chain/connect";

/**
 * Connects user's MetaMask wallet to the application
 * 
 * Handles the complete wallet connection flow including MetaMask integration,
 * GalaChain address retrieval, user registration checking, and automatic
 * registration if needed.
 * 
 * @param metamaskSupport - Reactive reference indicating MetaMask availability
 * @param metamaskClient - BrowserConnectClient instance for wallet operations
 * @param walletAddress - Reactive reference to store the connected wallet address
 * @param isConnected - Reactive reference to track connection state
 * 
 * @example
 * ```typescript
 * const metamaskSupport = ref(!!window.ethereum);
 * const client = new BrowserConnectClient();
 * const address = ref('');
 * const connected = ref(false);
 * 
 * await connectWallet(metamaskSupport, client, address, connected);
 * ```
 * 
 * @public
 */
export async function connectWallet(
  metamaskSupport: Ref<boolean, boolean>,
  metamaskClient: BrowserConnectClient,
  walletAddress: Ref<string, string>,
  isConnected: Ref<boolean, boolean>
) {
  if (!metamaskSupport.value) {
    return;
  }

  try {
    await metamaskClient.connect();
    walletAddress.value = metamaskClient.galaChainAddress;

    // Check registration
    try {
      await checkRegistration(walletAddress);
    } catch (e) {
      console.log(`registration check failed: ${e}. Attempting to register user: ${walletAddress}.`);
      await registerUser(metamaskClient, walletAddress);
    }

    isConnected.value = true;
  } catch (err) {
    console.error("Error connecting wallet:", err);
  }
}

/**
 * Checks if a wallet address is registered with GalaChain
 * 
 * Queries the GalaChain registration service to verify if the provided
 * wallet address has been properly registered and has an associated public key.
 * 
 * @param walletAddress - Reactive reference containing the wallet address to check
 * @throws Error if user is not registered or registration check fails
 * @internal
 */
export async function checkRegistration(walletAddress: Ref<string, string>) {
  const response = await fetch(
    `${import.meta.env.VITE_BURN_GATEWAY_PUBLIC_KEY_API}/GetPublicKey`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ user: walletAddress.value }),
    },
  );
  if (!response.ok) throw new Error("User not registered");
}

export async function registerUser(metamaskClient: BrowserConnectClient, walletAddress: Ref<string, string>) {
  const publicKey = await metamaskClient.getPublicKey();
  await fetch(`${import.meta.env.VITE_GALASWAP_API}/CreateHeadlessWallet`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ publicKey: publicKey.publicKey }),
  });
}
