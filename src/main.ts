import { createApp } from "vue";
import App from "./App.vue";
import { getCurrentWindow } from '@tauri-apps/api/window';
import { onShow, setUp } from "./windowApi/index"
const currentWin = getCurrentWindow();
(async () => {
  await setUp(currentWin);
  await onShow(currentWin);
})()
currentWin.setResizable(false);
currentWin.setAlwaysOnTop(true);
createApp(App).mount("#app");

