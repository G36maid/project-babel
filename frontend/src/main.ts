import { createPinia } from "pinia";
import { createApp } from "vue";
import "@erfanmola/televue/style.css";
import "./style.css";
import "./styles/telegram-theme.css";
import App from "./App.vue";
import router from "./router";

const app = createApp(App);
app.use(createPinia());
app.use(router);
app.mount("#app");
