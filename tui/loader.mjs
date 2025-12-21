import { fileURLToPath } from 'url';
import path from 'path';

export async function load(url, context, nextLoad) {
  if (url.endsWith('.scm') || url.endsWith('.wasm')) {
    let relativePath;
    if (context.parentURL) {
       const parentPath = path.dirname(fileURLToPath(context.parentURL));
       const childPath = fileURLToPath(url);
       relativePath = "./" + path.relative(parentPath, childPath);
    } else {
       relativePath = fileURLToPath(url);
    }
    
    return {
      format: 'module',
      shortCircuit: true,
      source: `export default ${JSON.stringify(relativePath)};`,
    };
  }

  return nextLoad(url, context);
}
