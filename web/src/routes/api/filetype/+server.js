import { exec } from 'child_process';
import { promisify } from 'util';
import path from 'path';

const execAsync = promisify(exec);
const ROOT_DIRECTORY = '/nethack';

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
    const { stdout } = await execAsync(`ttyrec-parser -b "${safePath}"`);
    return new Response(JSON.stringify({ type: stdout.trim() }), { status: 200 });
  } catch (err) {
    return new Response(JSON.stringify({ error: err.message }), { status: 500 });
  }
}
