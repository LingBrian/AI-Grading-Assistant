import { createRouter, createWebHashHistory } from 'vue-router'
import FloatingToolbar from '@/components/FloatingToolbar.vue'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      name: 'toolbar',
      component: FloatingToolbar
    },
    {
      path: '/screenshot',
      name: 'screenshot',
      component: () => import('@/components/ScreenshotOverlay.vue')
    }
  ]
})

export default router
