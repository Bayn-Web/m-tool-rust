<template>
  <div onsubmit="return false;" class="autocomplete-container">
    <div class="autocomplete">
      <input autocomplete="off" ref="input" @keydown.enter.prevent="search()" @keydown.up.prevent="chooseLi(true)"
        @keydown.down="chooseLi(false)" v-model="inputValue" id="drag-region" autofocus class="autocomplete-input"
        placeholder="Search in mTool" />
    </div>
    <ul id="autocomplete-results" class="autocomplete-results">
      <li v-for="result in results" :class='result.isSelected ? "autocomplete-result selected" : "autocomplete-result"'>
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

type item = { value: string; label: string };
const inputValue = ref("");
let data: item[] = [];
let results = ref<(item & { isSelected: boolean })[]>([]);
let selectedData: item = { value: "", label: "" };
let input = ref();
const currentWin = getCurrentWindow();

watch(inputValue, async (newValue) => {
  // dev
  const rowJSON = await invoke("get_json_data");
  data = JSON.parse(rowJSON as string);
  // dev__end
  results.value = data.filter((item) =>
    item.label.toLowerCase().includes(newValue.toLowerCase())
  ).map(res => { return { ...res, isSelected: false } })
  console.log(data)

  if (inputValue.value == "") {
    results.value = [];
    selectedData = { value: "", label: "" };
  } else {
    selectedData = results.value[0];
    results.value[0].isSelected = true;
    if (results.value.some(res => res.value === inputValue.value)) {
      input.value!.classList.add("success");
    } else {
      input.value!.classList.remove("success");
    }
  }
  currentWin.setSize(new LogicalSize(800, results.value.length * 55 + 40));
})

// file dragin
currentWin.onDragDropEvent(async (event) => {
  if (event.payload.type === "drop") {
    await invoke("upload_file_path", { path: event.payload.paths[0] });
    data = await invoke("get_json_data");
  }
});

onMounted(async () => {
  // able to drag
  document.getElementById('drag-region')?.addEventListener('mousedown', async () => {
    currentWin.startDragging();
  });
  setTimeout(async () => {
    await onShow(currentWin);
    await currentWin.setFocus();
    const rowJSON = await invoke("get_json_data");
    data = JSON.parse(rowJSON as string);
  }, 0);
});

async function runExec(cmd: string) {
  try {
    await invoke("start_cmd", { cmd });
    input.value!.select();
    onHide(currentWin);
  } catch (error) {
    console.error(error);
  }
}
const search = () => {
  if (input.value!.classList.contains("success")) {
    runExec(results.value.find((item) => item.value === inputValue.value)!.label);
    onHide(currentWin);
  } else {
    inputValue.value = selectedData.value;
  }
};

const chooseLi = (isUp: boolean) => {
  console.log(1)
  if (results.value.length > 0) {
    let index = results.value.findIndex((item, index, arr) => {
      if (item.isSelected) {
        arr[index].isSelected = false;
        return true;
      }
    })
    if (isUp) {
      index = index == 0 ? 0 : index - 1;
    } else {
      index = index == results.value.length-1 ? results.value.length-1 : index + 1;
    }
    results.value[index].isSelected = true;
    selectedData = results.value[index];
  }
}
</script>
