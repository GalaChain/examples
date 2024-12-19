/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_BURN_GATEWAY_API: string
  readonly VITE_FAUCET_GATEWAY_API: string
  readonly VITE_FAUCET_MULTIPLIER: string
  readonly VITE_FAUCET_ADMIN_PRIVATE_KEY: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}