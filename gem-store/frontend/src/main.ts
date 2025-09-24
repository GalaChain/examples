import { createApp } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import App from './App.vue'
import './style.css'

// Import components
import WalletConnect from './components/WalletConnect.vue'
import GemStore from './components/GemStore.vue'
import TransactionHistory from './components/TransactionHistory.vue'

// Define routes
const routes = [
  { path: '/', component: WalletConnect },
  { path: '/store', component: GemStore },
  { path: '/history', component: TransactionHistory }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

const app = createApp(App)
app.use(router)
app.mount('#app')
