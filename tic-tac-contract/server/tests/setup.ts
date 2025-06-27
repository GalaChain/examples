import { config } from 'dotenv';

// Load test environment variables
config({ path: '.env.test' });

// Set test timeouts
jest.setTimeout(10000);

// Mock environment variables for tests
process.env.NODE_ENV = 'test';
process.env.GALACHAIN_API_URL = process.env.GALACHAIN_API_URL || 'http://localhost:3000';