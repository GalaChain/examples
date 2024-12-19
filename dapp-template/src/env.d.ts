/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_BURN_GATEWAY_API: string
  readonly VITE_BURN_GATEWAY_PUBLIC_KEY_API: string
  readonly VITE_GALASWAP_API: string
  readonly VITE_PROJECT_ID: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}