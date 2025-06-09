<script>
  import { onMount } from 'svelte';
  let currentDir = '.';
  let files = [];
  let selectedFile = '';
  let fileType = '';

  async function fetchFiles() {
    const res = await fetch(`/api/list?dir=${encodeURIComponent(currentDir)}`);
    files = await res.json();
    selectedFile = '';
    fileType = '';
  }

  async function selectFile(fileName) {
    selectedFile = `${currentDir}/${fileName}`;
    const res = await fetch(`/api/filetype?path=${encodeURIComponent(selectedFile)}`);
    const data = await res.json();
    fileType = data.type;
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

  onMount(fetchFiles);
</script>

<main class="p-6 max-w-5xl mx-auto">
  <h1 class="text-2xl font-bold mb-4">📂 File Browser (/nethack)</h1>

  <div class="flex items-center mb-4">
    <button on:click={goUp} class="bg-blue-500 text-white px-2 py-1 rounded mr-2">⬆️ Up</button>
    <span class="text-gray-600">Current: {currentDir}</span>
  </div>

  <div class="flex space-x-4">
    <!-- Left pane: Directory listing -->
    <div class="w-1/2 border rounded p-2 space-y-1 overflow-y-auto max-h-[600px]">
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
          </li>
        {/each}
      </ul>
    </div>

    <!-- Right pane: File details -->
    <div class="w-1/2 border rounded p-4 bg-gray-50 min-h-[200px]">
      <h2 class="font-semibold mb-2">File Details</h2>
      {#if selectedFile}
        <p class="mb-2"><strong>Path:</strong> {selectedFile}</p>
        <p><strong>Type:</strong> {fileType}</p>
      {:else}
        <p class="text-gray-500">Select a file to view its type.</p>
      {/if}
    </div>
  </div>
</main>
