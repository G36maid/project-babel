import { createApp } from 'vue'
import { createPinia } from 'pinia'
import TeleVue from '@erfanmola/televue'
import './style.css'
import './styles/telegram-theme.css'
import App from './App.vue'
import router from './router'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.use(TeleVue)
app.mount('#app')
