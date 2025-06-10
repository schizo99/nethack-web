import { exec } from 'child_process';
import { promisify } from 'util';
import path from 'path';

const execAsync = promisify(exec);
const ROOT_DIRECTORY = import.meta.env.VITE_ROOT_DIRECTORY || '/nethack';

function resolveSafePath(relativePath) {
  const safePath = path.resolve(ROOT_DIRECTORY, relativePath);
  if (!safePath.startsWith(ROOT_DIRECTORY)) {
    throw new Error('Access denied');
  }
  return safePath;
}

export async function GET({ url }) {
  const relativePath = url.searchParams.get('path');
  if (!relativePath) {
    return new Response(JSON.stringify({ error: 'Missing path parameter' }), { status: 400 });
  }

  try {
    const safePath = resolveSafePath(relativePath);
    const { stdout } = await execAsync(`ttyrec-parser "${safePath}"`);

    if (!stdout.includes('You die...')) {
      return new Response(JSON.stringify({ error: 'File validation failed: missing "You die..." in output' }), { status: 400 });
    }
    return new Response(JSON.stringify({ type: stdout.trim() }), { status: 200 });
  } catch (err) {
    return new Response(JSON.stringify({ error: err.message }), { status: 500 });
  }
}
