import { access, readFile, readdir } from "node:fs/promises";
import { relative, resolve } from "node:path";
import { bodyHeadingIds, loadDiskContent } from "@scripts/lib/content-files";

function invariant(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

async function walk(directory: string): Promise<string[]> {
  const entries = await readdir(directory, { withFileTypes: true });
  const nested = await Promise.all(
    entries.map(async (entry) => {
      const path = resolve(directory, entry.name);
      return entry.isDirectory() ? walk(path) : [path];
    }),
  );
  return nested.flat();
}

const root = process.cwd();
const content = await loadDiskContent();
const registrySource = await readFile(
  resolve(root, "src", "lib", "content", "registry.ts"),
  "utf8",
);
const registeredMdx = new Set(
  [...registrySource.matchAll(/from "@content\/([^"]+\.mdx)"/g)].map((match) =>
    match[1].replaceAll("/", "\\"),
  ),
);

for (const entry of content) {
  const relativePath = relative(resolve(root, "content"), entry.filePath);
  invariant(registeredMdx.has(relativePath), `MDX file is missing from static registry: ${relativePath}`);

  const ids = bodyHeadingIds(entry.source);
  invariant(new Set(ids).size === ids.length, `${relativePath} contains duplicate heading IDs.`);

  const heroPath = resolve(root, "public", entry.frontmatter.hero.src.slice(1));
  await access(heroPath);
}

invariant(
  registeredMdx.size === content.length,
  `Registry imports ${registeredMdx.size} MDX files, but disk contains ${content.length}.`,
);

const sourceFiles = (
  await Promise.all(
    ["src", "scripts", "tests"].map(async (directory) => {
      try {
        return await walk(resolve(root, directory));
      } catch {
        return [];
      }
    }),
  )
)
  .flat()
  .filter((path) => /\.(?:ts|tsx|mjs)$/.test(path));

const clientFiles: string[] = [];
for (const filePath of sourceFiles) {
  const source = await readFile(filePath, "utf8");
  const name = relative(root, filePath).replaceAll("\\", "/");
  invariant(!/from\s+["']\.\.\//.test(source), `${name} imports through ../.`);
  invariant(!/import\(\s*["']\.\.\//.test(source), `${name} dynamically imports through ../.`);
  invariant(!/[A-Za-z]:\\agent-kernel/i.test(source), `${name} contains a parent-runtime path.`);
  invariant(!/https?:\/\/(?:fonts|images)\./i.test(source), `${name} hotlinks a font or image.`);
  if (/^["']use client["'];/m.test(source)) {
    clientFiles.push(name);
  }
}

const allowedClientFiles = new Set([
  "src/app/[locale]/error.tsx",
  "src/app/global-error.tsx",
  "src/components/navigation/mobile-navigation.tsx",
]);
for (const file of clientFiles) {
  invariant(allowedClientFiles.has(file), `Unexpected client component: ${file}`);
}
invariant(clientFiles.length === allowedClientFiles.size, "Expected only navigation and errors to be client-side.");

console.log(
  `Link and boundary checks passed: ${registeredMdx.size} static MDX imports, ${clientFiles.length} intentional client files.`,
);
