import { BrowserConnectClient } from '@gala-chain/connect'

export interface ConnectionResult {
  address: string
  provider: any
}

export const walletService = {
  client: null as BrowserConnectClient | null,
  
  async connect(): Promise<ConnectionResult> {
    this.client = new BrowserConnectClient()
    const connected = await this.client.connect()
    
    if (connected && this.client.galaChainAddress) {
      return {
        address: this.client.galaChainAddress,
        provider: this.client // The client itself acts as the provider
      }
    }
    throw new Error('Failed to connect wallet')
  },
  
  async signMessage(message: string) {
    if (!this.client) throw new Error('Wallet not connected')
    
    const timestamp = Date.now()
    const fullMessage = `Sign this message to authenticate:\n${message}\nTimestamp: ${timestamp}`
    
    const signature = await this.client.personalSign(fullMessage)
    return { signature, timestamp }
  },
  
  async getBalance(address: string) {
    const response = await fetch(`${import.meta.env.VITE_BURN_GATEWAY_API}/balance/${address}`)
    const data = await response.json()
    return data.balance
  },
  
  disconnect() {
    if (this.client) {
      this.client.disconnect()
      this.client = null
    }
  }
}
