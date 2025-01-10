/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_GATEWAY_TOKEN_API: string
  readonly VITE_GALACONNECT_PUBLIC_KEY_API: string
  readonly VITE_GALACONNECT_API: string
  readonly VITE_PROJECT_ID: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}