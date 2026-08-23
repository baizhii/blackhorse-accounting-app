import { createApp } from "vue";
import { createPinia } from "pinia";
import ElementPlus from "element-plus";
import "element-plus/dist/index.css";
import "element-plus/theme-chalk/dark/css-vars.css";
import "./assets/fonts/noto-sans-sc.css";
import zhCn from "element-plus/es/locale/lang/zh-cn";
import App from "./App.vue";

// 跟随系统浅色 / 深色模式：Element Plus 暗色样式依赖 html 上的 .dark 类
const darkQuery = window.matchMedia("(prefers-color-scheme: dark)");
function applySystemTheme() {
  document.documentElement.classList.toggle("dark", darkQuery.matches);
}
applySystemTheme();
darkQuery.addEventListener("change", applySystemTheme);

createApp(App).use(createPinia()).use(ElementPlus, { locale: zhCn }).mount("#app");
