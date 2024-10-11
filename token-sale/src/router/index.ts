import { createRouter, createWebHistory } from 'vue-router'
import Home from '../views/home.vue'
import Sale from '../views/sale.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: Home
    },
    {
      path: '/sale/:tokenSaleId',
      name: 'sale',
      component: Sale
    }
  ]
})

export default router
