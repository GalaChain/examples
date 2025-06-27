/**
 * @fileoverview Tests for wallet connection functionality
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ref } from 'vue';
import { connectWallet } from '../src/connect';
import { BrowserConnectClient } from '@gala-chain/connect';

// Mock the BrowserConnectClient
vi.mock('@gala-chain/connect', () => ({
  BrowserConnectClient: vi.fn().mockImplementation(() => ({
    connect: vi.fn(),
    galaChainAddress: 'test-address-123',
    disconnect: vi.fn(),
    isConnected: false
  }))
}));

// Mock global fetch
global.fetch = vi.fn();

describe('connectWallet', () => {
  let metamaskSupport: ReturnType<typeof ref<boolean>>;
  let metamaskClient: BrowserConnectClient;
  let walletAddress: ReturnType<typeof ref<string>>;
  let isConnected: ReturnType<typeof ref<boolean>>;

  beforeEach(() => {
    metamaskSupport = ref(true);
    metamaskClient = new BrowserConnectClient();
    walletAddress = ref('');
    isConnected = ref(false);

    vi.clearAllMocks();
  });

  it('should connect wallet successfully when metamask is supported', async () => {
    // Mock successful connection
    vi.mocked(metamaskClient.connect).mockResolvedValueOnce(undefined);
    
    // Mock successful registration check
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      json: async () => ({ registered: true })
    } as Response);

    await connectWallet(metamaskSupport, metamaskClient, walletAddress, isConnected);

    expect(metamaskClient.connect).toHaveBeenCalled();
    expect(walletAddress.value).toBe('test-address-123');
  });

  it('should not attempt connection when metamask is not supported', async () => {
    metamaskSupport.value = false;

    await connectWallet(metamaskSupport, metamaskClient, walletAddress, isConnected);

    expect(metamaskClient.connect).not.toHaveBeenCalled();
    expect(walletAddress.value).toBe('');
  });

  it('should handle connection errors gracefully', async () => {
    const connectionError = new Error('Connection failed');
    vi.mocked(metamaskClient.connect).mockRejectedValueOnce(connectionError);

    // Should not throw, should handle gracefully
    await expect(connectWallet(metamaskSupport, metamaskClient, walletAddress, isConnected))
      .resolves.toBeUndefined();

    expect(walletAddress.value).toBe('');
  });

  it('should handle registration check failure', async () => {
    vi.mocked(metamaskClient.connect).mockResolvedValueOnce(undefined);
    
    // Mock failed registration check
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: false,
      status: 404,
      text: async () => 'User not registered'
    } as Response);

    await connectWallet(metamaskSupport, metamaskClient, walletAddress, isConnected);

    expect(metamaskClient.connect).toHaveBeenCalled();
    expect(walletAddress.value).toBe('test-address-123');
    // Should still set address even if registration check fails
  });

  it('should handle network errors during registration check', async () => {
    vi.mocked(metamaskClient.connect).mockResolvedValueOnce(undefined);
    
    // Mock network error
    vi.mocked(fetch).mockRejectedValueOnce(new Error('Network error'));

    await connectWallet(metamaskSupport, metamaskClient, walletAddress, isConnected);

    expect(metamaskClient.connect).toHaveBeenCalled();
    expect(walletAddress.value).toBe('test-address-123');
  });
});

describe('checkRegistration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should check registration status successfully', async () => {
    const mockAddress = ref('test-wallet-address');
    
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      json: async () => ({ registered: true, publicKey: 'test-public-key' })
    } as Response);

    // Import and test checkRegistration if it's exported
    // For now, we test it indirectly through connectWallet
    const metamaskSupport = ref(true);
    const metamaskClient = new BrowserConnectClient();
    const walletAddress = ref('');
    const isConnected = ref(false);

    vi.mocked(metamaskClient.connect).mockResolvedValueOnce(undefined);

    await connectWallet(metamaskSupport, metamaskClient, walletAddress, isConnected);

    expect(fetch).toHaveBeenCalledWith(
      expect.stringContaining('/identities/public-key/'),
      expect.objectContaining({
        method: 'GET'
      })
    );
  });

  it('should handle registration check with unregistered user', async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: false,
      status: 404
    } as Response);

    const metamaskSupport = ref(true);
    const metamaskClient = new BrowserConnectClient();
    const walletAddress = ref('');
    const isConnected = ref(false);

    vi.mocked(metamaskClient.connect).mockResolvedValueOnce(undefined);

    await connectWallet(metamaskSupport, metamaskClient, walletAddress, isConnected);

    // Should not throw error for unregistered user
    expect(walletAddress.value).toBe('test-address-123');
  });
});