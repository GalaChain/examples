import { config } from '@vue/test-utils';

// Global test setup for Vue Testing Library
config.global.stubs = {
  // Stub any global components if needed
};

// Mock environment variables for tests
Object.defineProperty(window, 'ethereum', {
  value: {
    request: vi.fn(),
    on: vi.fn(),
    removeListener: vi.fn(),
  },
  writable: true,
});

// Mock console methods to reduce noise in tests
global.console = {
  ...console,
  warn: vi.fn(),
  error: vi.fn(),
};