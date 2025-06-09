import fs from 'fs/promises';
import path from 'path';

const ROOT_DIRECTORY = '/nethack';

function resolveSafePath(relativePath) {
  const safePath = path.resolve(ROOT_DIRECTORY, relativePath);
  if (!safePath.startsWith(ROOT_DIRECTORY)) {
    throw new Error('Access denied');
  }
  return safePath;
}

export async function GET({ url }) {
  const relativeDir = url.searchParams.get('dir') || '.';
  const sortParam = url.searchParams.get('sort') || 'mtime'; // default
  const orderParam = url.searchParams.get('order') || 'desc'; // default

  try {
    const dirPath = resolveSafePath(relativeDir);
    const entries = await fs.readdir(dirPath, { withFileTypes: true });

    const fileInfos = await Promise.all(
      entries.map(async (entry) => {
        const fullPath = path.join(dirPath, entry.name);
        const stat = await fs.stat(fullPath);
        return {
          name: entry.name,
          isDirectory: entry.isDirectory(),
          mtimeMs: stat.mtimeMs,
          size: stat.size,
        };
      })
    );

    const filtered = fileInfos.filter((f) => {
      if (f.isDirectory && f.name === 'dumplog') {
        return false;
      }
      return f.isDirectory || f.size >= 10 * 1024;
    });

    const sorted = filtered.sort((a, b) => {
      let result = 0;
      if (sortParam === 'name') {
        result = a.name.localeCompare(b.name);
      } else if (sortParam === 'size') {
        result = a.size - b.size;
      } else {
        // default: mtime
        result = a.mtimeMs - b.mtimeMs;
      }

      return orderParam === 'asc' ? result : -result;
    });

    return new Response(JSON.stringify(sorted), { status: 200 });
  } catch (err) {
    return new Response(JSON.stringify({ error: err.message }), { status: 500 });
  }
}
