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

  try {
    const dirPath = resolveSafePath(relativeDir);
    const files = await fs.readdir(dirPath, { withFileTypes: true });

    return new Response(
      JSON.stringify(
        files.map((f) => ({
          name: f.name,
          isDirectory: f.isDirectory(),
        }))
      ),
      { status: 200 }
    );
  } catch (err) {
    return new Response(JSON.stringify({ error: err.message }), { status: 500 });
  }
}
