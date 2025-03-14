import { Ref } from "vue";
import { BrowserConnectClient } from "@gala-chain/connect";

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
