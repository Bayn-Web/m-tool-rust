import { register, unregister } from '@tauri-apps/plugin-global-shortcut';
import { Window } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';

export const onShow = async (currentWin: Window) => {
  await register('Esc', async () => {
    if (await currentWin.isFocused())
      invoke('save_position')
      currentWin.destroy();
  });
  await currentWin.show();
  await currentWin.setFocus();
  document.querySelector("input")!.focus();
}

export const onHide = async (currentWin: Window) => {
  await unregister('Esc');
  await currentWin.hide();
}

export const setUp = async (currentWin: Window) => {
  await register('Alt+M', async (e) => {
    if (e.state == "Pressed")
      if (await currentWin.isFocused())
        onHide(currentWin);
      else
        onShow(currentWin);
  });
}