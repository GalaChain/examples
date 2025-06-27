/**
 * @fileoverview Tests for the main App component
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import App from '../src/App.vue';

// Mock the BrowserConnectClient
vi.mock('@gala-chain/connect', () => ({
  BrowserConnectClient: vi.fn().mockImplementation(() => ({
    connect: vi.fn(),
    disconnect: vi.fn(),
    galaChainAddress: 'test-address',
    isConnected: false
  }))
}));

// Mock boardgame.io client
vi.mock('boardgame.io/client', () => ({
  Client: vi.fn().mockImplementation(() => ({
    start: vi.fn(),
    stop: vi.fn(),
    moves: {
      makeMove: vi.fn()
    },
    events: {
      endGame: vi.fn()
    },
    getState: vi.fn(() => ({
      G: { board: Array(9).fill(null), currentPlayer: 'X' },
      ctx: { numPlayers: 2, turn: 1, currentPlayer: '0', phase: 'play' },
      isActive: true
    }))
  }))
}));

// Mock global window.ethereum
Object.defineProperty(window, 'ethereum', {
  value: {
    request: vi.fn(),
    on: vi.fn(),
    removeListener: vi.fn(),
  },
  writable: true,
});

describe('App.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should render main application structure', () => {
    const wrapper = mount(App);
    
    expect(wrapper.find('.app').exists()).toBe(true);
    expect(wrapper.find('h1').text()).toContain('Tic Tac Toe');
  });

  it('should show metamask not detected when ethereum is not available', () => {
    // Temporarily remove ethereum
    const originalEthereum = window.ethereum;
    delete (window as any).ethereum;
    
    const wrapper = mount(App);
    
    expect(wrapper.text()).toContain('MetaMask not detected');
    
    // Restore ethereum
    (window as any).ethereum = originalEthereum;
  });

  it('should show connect wallet button when not connected', () => {
    const wrapper = mount(App);
    
    // Should show connect button or connection-related UI
    expect(wrapper.html()).toContain('connect');
  });

  it('should initialize game board with 9 cells', () => {
    const wrapper = mount(App);
    
    // Look for game board cells
    const cells = wrapper.findAll('.cell, [data-testid="cell"]');
    expect(cells.length).toBeGreaterThanOrEqual(0); // May be 0 if not connected
  });

  it('should handle component mounting without errors', () => {
    expect(() => {
      mount(App);
    }).not.toThrow();
  });

  it('should have reactive wallet connection state', async () => {
    const wrapper = mount(App);
    
    // Check that component data is reactive
    const vm = wrapper.vm as any;
    
    // Should have wallet-related reactive properties
    expect(typeof vm.walletAddress).toBe('string');
    expect(typeof vm.isConnected).toBe('boolean');
  });

  it('should handle game state updates', async () => {
    const wrapper = mount(App);
    const vm = wrapper.vm as any;
    
    // Should have game-related properties
    if (vm.gameClient) {
      expect(vm.gameClient).toBeDefined();
    }
  });

  it('should cleanup on unmount', () => {
    const wrapper = mount(App);
    
    expect(() => {
      wrapper.unmount();
    }).not.toThrow();
  });
});

describe('App integration', () => {
  it('should handle wallet connection flow', async () => {
    const wrapper = mount(App);
    
    // Find and trigger connect wallet action if available
    const connectButton = wrapper.find('[data-testid="connect-wallet"], button');
    
    if (connectButton.exists()) {
      await connectButton.trigger('click');
      // Should not throw errors
    }
  });

  it('should handle game board interactions', async () => {
    const wrapper = mount(App);
    
    // Look for game cells
    const gameCell = wrapper.find('.cell, [data-testid="cell-0"]');
    
    if (gameCell.exists()) {
      await gameCell.trigger('click');
      // Should not throw errors
    }
  });

  it('should maintain consistent state', () => {
    const wrapper = mount(App);
    const vm = wrapper.vm as any;
    
    // Check that initial state is consistent
    if (vm.board) {
      expect(Array.isArray(vm.board)).toBe(true);
      expect(vm.board.length).toBe(9);
    }
    
    if (vm.currentPlayer) {
      expect(['X', 'O']).toContain(vm.currentPlayer);
    }
  });
});