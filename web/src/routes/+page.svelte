<script>
  import { onMount } from "svelte";
  let currentDir = ".";
  let files = [];
  let selectedFile = "";
  let fileTypeLines = [];
  let showModal = false;
  let sort = "mtime"; // default sort
  let order = "desc"; // default order
  let orderText = order === "asc" ? "Ascending ▲" : "Descending ▼";
  async function fetchFiles() {
    const res = await fetch(
      `/api/list?dir=${encodeURIComponent(currentDir)}&sort=${encodeURIComponent(sort)}&order=${encodeURIComponent(order)}`,
    );
    files = await res.json();
    selectedFile = "";
    fileTypeLines = [];
    showModal = false;
  }

  async function selectFile(fileName) {
    selectedFile = `${currentDir}/${fileName}`;
    const res = await fetch(
      `/api/filetype?path=${encodeURIComponent(selectedFile)}`,
    );
    const data = await res.json();
    if (res.status !== 200) {
      fileTypeLines = [data.error];
    } else {
      fileTypeLines = data.type.split(/\r?\n/);
    }
    showModal = true;
  }

  function openDirectory(dirName) {
    currentDir = `${currentDir}/${dirName}`;
    fetchFiles();
  }

  function goUp() {
    const parts = currentDir.split("/");
    parts.pop();
    currentDir = parts.length ? parts.join("/") : ".";
    fetchFiles();
  }

  function closeModal() {
    showModal = false;
  }

  function changeSort(event) {
    sort = event.target.value;
    fetchFiles();
  }

  function changeOrder(event) {
    order = order === "asc" ? "desc" : "asc";
    orderText = order === "asc" ? "Ascending ▲" : "Descending ▼";
    fetchFiles();
  }

  onMount(fetchFiles);
</script>

<main class="p-6 max-w-5xl mx-auto">
  <div class="flex items-center mb-4 space-x-4 flex-wrap">
    <h1 class="pt-4 text-2xl font-bold mb-4">📂 File Browser</h1>
    <!-- Sort dropdown -->
    <div class="ml-auto flex space-x-2 items-center">
      <label class="text-gray-600" for="sortSelect">Sort by:</label>
      <select
        id="sortSelect"
        bind:value={sort}
        onchange={changeSort}
        class="border rounded px-2 py-1 bg-none"
      >
        <option value="mtime">Modified Time</option>
        <option value="size">Size</option>
        <option value="name">Name</option>
      </select>

      <input
        id="orderInput"
        type="button"
        bind:value={orderText}
        onclick={changeOrder}
        class="btn border rounded px-2 py-1 bg-none"
      />
    </div>
  </div>

  <div class="flex space-x-4">
    <!-- Left pane: Directory listing -->
    <div
      class="w-full border rounded p-2 space-y-1 overflow-y-auto max-h-[600px]"
    >
      <div class="flex items-center mb-2">
        <button onclick={goUp} class="bg-blue-400 text-white px-2 py-1 rounded">
          ⬆️ Up
        </button>
        <h2 class="pl-4 font-semibold">Contents of {currentDir}</h2>
      </div>
      <div
        class="grid grid-cols-8 font-semibold text-gray-500 text-sm mb-2 px-2"
      >
        <span class="col-start-1 col-end-4">Filename</span>
        <span class="col-start-6 col-end-8 text-right">Last Modified</span>
        <span class="col-start-8 col-end-8 text-right">Size</span>
      </div>
      <ul>
        {#each files as file}
          <li
            class="grid grid-cols-8 items-center hover:bg-gray-100 p-1 rounded"
          >
            <span class="col-start-1 col-end-4">
              {#if file.isDirectory}
                📁 <button
                  onclick={() => openDirectory(file.name)}
                  class="text-blue-600 underline">{file.name}</button
                >
              {:else}
                📄 <button
                  onclick={() => selectFile(file.name)}
                  class="text-black">{file.name}</button
                >
              {/if}
            </span>
            <span
              class="col-start-6 col-end-8 text-gray-500 text-sm text-right"
            >
              {#if !file.isDirectory}{new Date(
                  file.mtimeMs,
                ).toLocaleString()}{/if}
            </span>
            <span
              class="col-start-8 col-end-8 text-gray-500 text-sm text-right"
            >
              {#if !file.isDirectory}{(file.size / 1024).toFixed(1)} KB{/if}
            </span>
          </li>
        {/each}
      </ul>
    </div>
  </div>

  <!-- Modal -->
  {#if showModal}
    <div
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      role="dialog"
      aria-modal="true"
      tabindex="0"
      onclick={closeModal}
      onkeydown={(e) => {
        if (e.key === "Escape") closeModal();
      }}
    >
      <div
        class="bg-white rounded-lg p-6 max-w-4xl w-full shadow-lg"
        aria-modal="true"
        role="dialog"
        tabindex="0"
        onclick={(event) => {
          event.stopPropagation();
        }}
        onkeydown={() => {}}
      >
        <h2 class="text-xl font-semibold mb-4">File Details</h2>
        <p class="mb-2"><strong>Path:</strong> {selectedFile}</p>
        <p class="mb-2"><strong>Type:</strong></p>
        <ul
          class="list-disc list-inside mb-4 max-h-96 overflow-y-auto text-sm whitespace-pre-wrap break-words"
        >
          {#each fileTypeLines as line}
            <li>{line}</li>
          {/each}
        </ul>
        <div class="flex justify-end">
          <button
            onclick={closeModal}
            class="bg-blue-500 text-white px-4 py-2 rounded">Close</button
          >
        </div>
      </div>
    </div>
  {/if}
</main>
