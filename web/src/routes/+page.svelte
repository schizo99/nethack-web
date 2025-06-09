<script>
  import { onMount } from 'svelte';
  let currentDir = '.';
  let files = [];
  let selectedFile = '';
  let fileTypeLines = [];
  let showModal = false;
  let sort = 'mtime'; // default sort
  let order = 'desc'; // default order


  async function fetchFiles() {
  const res = await fetch(
    `/api/list?dir=${encodeURIComponent(currentDir)}&sort=${encodeURIComponent(sort)}&order=${encodeURIComponent(order)}`
  );
  files = await res.json();
  selectedFile = '';
  fileTypeLines = [];
  showModal = false;
}

  async function selectFile(fileName) {
    selectedFile = `${currentDir}/${fileName}`;
    const res = await fetch(`/api/filetype?path=${encodeURIComponent(selectedFile)}`);
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
    const parts = currentDir.split('/');
    parts.pop();
    currentDir = parts.length ? parts.join('/') : '.';
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
  console.log(event.target.value)
  order = event.target.value;
  fetchFiles();
}


  onMount(fetchFiles);
</script>

<main class="p-6 max-w-5xl mx-auto">
  <h1 class="text-2xl font-bold mb-4">📂 File Browser (/nethack)</h1>

  <div class="flex items-center mb-4 space-x-4 flex-wrap">
  <button on:click={goUp} class="bg-blue-500 text-white px-2 py-1 rounded">⬆️ Up</button>
  <span class="text-gray-600">Current: {currentDir}</span>

  <!-- Sort dropdown -->
  <div class="ml-auto flex space-x-2 items-center">
    <label class="text-gray-600">Sort by:</label>
    <select bind:value={sort} on:change={changeSort} class="border rounded px-2 py-1">
      <option value="mtime">Modified Time</option>
      <option value="size">Size</option>
      <option value="name">Name</option>
    </select>

    <label class="text-gray-600 ml-4">Order:</label>
    <select bind:value={order} on:change={changeOrder} class="border rounded px-2 py-1">
      <option value="desc">Descending</option>
      <option value="asc">Ascending</option>
    </select>
  </div>
</div>

  <div class="flex space-x-4">
    <!-- Left pane: Directory listing -->
    <div class="w-full border rounded p-2 space-y-1 overflow-y-auto max-h-[600px]">
      <h2 class="font-semibold mb-2">Contents of {currentDir}</h2>
      <ul>
        {#each files as file}
          <li class="flex items-center justify-between hover:bg-gray-100 p-1 rounded">
            <span>
              {#if file.isDirectory}
                📁 <button on:click={() => openDirectory(file.name)} class="text-blue-600 underline">{file.name}</button>
              {:else}
                📄 <button on:click={() => selectFile(file.name)} class="text-black">{file.name}</button>
              {/if}
            </span>
            <span class="text-gray-500 text-sm ml-2">
      {#if !file.isDirectory}{(file.size / 1024).toFixed(1)} KB{/if}
    </span>
          </li>
        {/each}
      </ul>
    </div>
  </div>

 <!-- Modal -->
{#if showModal}
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-white rounded-lg p-6 max-w-4xl w-full shadow-lg">
      <h2 class="text-xl font-semibold mb-4">File Details</h2>
      <p class="mb-2"><strong>Path:</strong> {selectedFile}</p>
      <p class="mb-2"><strong>Type:</strong></p>
      <ul class="list-disc list-inside mb-4 max-h-96 overflow-y-auto text-sm whitespace-pre-wrap break-words">
        {#each fileTypeLines as line}
          <li>{line}</li>
        {/each}
      </ul>
      <div class="flex justify-end">
        <button on:click={closeModal} class="bg-blue-500 text-white px-4 py-2 rounded">Close</button>
      </div>
    </div>
  </div>
{/if}

</main>
