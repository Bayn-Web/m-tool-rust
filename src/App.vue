<template>
  <div onsubmit="return false;" class="autocomplete-container">
    <div class="autocomplete">
      <input
        autocomplete="off"
        ref="input"
        @keydown.enter.prevent="search()"
        @keydown.up.prevent="chooseLi(true)"
        @keydown.down="chooseLi(false)"
        v-model="inputValue"
        id="drag-region"
        autofocus
        class="autocomplete-input"
        placeholder="Search in mTool"
      />
    </div>
    <ul id="autocomplete-results" class="autocomplete-results">
      <li
        v-for="result in results"
        :class="
          result.isSelected ? 'autocomplete-result selected' : 'autocomplete-result'
        "
      >
        <h3>{{ result.value }}</h3>
        <h5>{{ result.label }}</h5>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { onHide, onShow } from "./windowApi";
import { listen } from "@tauri-apps/api/event";

listen<boolean>("play_finish", (event) => {
  // 随机取一个
  playRandomSong();
});

type item = { value: string; label: string };
let musicList: item[] = [];
const inputValue = ref("");
let data: item[] = [];
let results = ref<(item & { isSelected: boolean })[]>([]);
let selectedData: item = { value: "", label: "" };
let input = ref();
const currentWin = getCurrentWindow();

watch(inputValue, async (newValue) => {
  // dev
  const rowJSON = await invoke("get_json_data");
  data = JSON.parse(rowJSON as string).concat(musicList);
  // dev__end
  results.value = data
    .filter((item) => item.label.toLowerCase().includes(newValue.toLowerCase()))
    .map((res) => {
      return { ...res, isSelected: false };
    });

  if (inputValue.value == "") {
    results.value = [];
    selectedData = { value: "", label: "" };
  } else {
    if (results.value.length > 0) {
      selectedData = results.value[0];
      results.value[0].isSelected = true;
      if (results.value.some((res) => res.value === inputValue.value)) {
        input.value!.classList.add("success");
      } else {
        input.value!.classList.remove("success");
      }
    }
  }
  currentWin.setSize(new LogicalSize(800, results.value.length * 55 + 40));
});

// file dragin
currentWin.onDragDropEvent(async (event) => {
  if (event.payload.type === "drop") {
    await invoke("upload_file_path", { path: event.payload.paths[0] });
    data = await invoke("get_json_data");
  }
});

onMounted(async () => {
  // able to drag
  document.getElementById("drag-region")?.addEventListener("mousedown", async () => {
    currentWin.startDragging();
  });
  if (localStorage.getItem("volume") == null) {
    localStorage.setItem("volume", 1);
  }
  setTimeout(async () => {
    await onShow(currentWin);
    await currentWin.setFocus();
    const rowJSON = await invoke("get_json_data");
    musicList = await invoke("get_songs");
    console.log(musicList);
    data = JSON.parse(rowJSON as string);
    data.concat(musicList);
  }, 0);
});

async function runExec(cmd: string) {
  try {
    if (cmd.endsWith("mp3")) {
      await invoke("play_song", {
        title: cmd,
        vol: parseFloat(localStorage.getItem("volume")),
      });
    } else {
      await invoke("start_cmd", { cmd });
    }
    input.value!.select();
    onHide(currentWin);
  } catch (error) {
    console.error(error);
  }
}
const playRandomSong = () => {
  invoke("play_song", {
    ...musicList[Math.floor(Math.random() * musicList.length)],
    vol: parseFloat(localStorage.getItem("volume")),
  });
};
const search = async () => {
  if (inputValue.value.startsWith("volume")) {
    localStorage.setItem(
      "volume",
      parseInt((inputValue.value as string).split(" ")[1]) / 100.0
    );
    invoke("set_volume", {
      vol: parseFloat(localStorage.getItem("volume")),
    });
    return;
  }
  if (inputValue.value == "stop") {
    invoke("pause_song");
    return;
  }
  if (inputValue.value == "music") {
    musicList = await invoke("get_songs");
    playRandomSong();
    return;
  }
  if (input.value!.classList.contains("success")) {
    runExec(results.value.find((item) => item.value === inputValue.value)!.label);
    onHide(currentWin);
  } else {
    inputValue.value = selectedData.value;
  }
};

const chooseLi = (isUp: boolean) => {
  if (results.value.length > 0) {
    let index = results.value.findIndex((item, index, arr) => {
      if (item.isSelected) {
        arr[index].isSelected = false;
        return true;
      }
    });
    if (isUp) {
      index = index == 0 ? 0 : index - 1;
    } else {
      index = index == results.value.length - 1 ? results.value.length - 1 : index + 1;
    }
    results.value[index].isSelected = true;
    selectedData = results.value[index];
  }
};
</script>
