#!/usr/bin/env node

/**
 * Checks if generated content is up-to-date with source files.
 * Generates to a temp directory and compares against committed content.
 * Exits with code 1 if any files are stale.
 */

import { mkdtempSync, readFileSync, rmSync, existsSync } from "node:fs";
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { dirname } from "node:path";

import { generateDocsPages } from "./generate-docs-pages.mjs";
import { generateWorkflowPages } from "./generate-workflow-pages.mjs";
import { generateReferencePages } from "./generate-reference-pages.mjs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const siteRoot = resolve(__dirname, "..");
const committedDir = join(siteRoot, "src", "content", "docs");

const tempDir = mkdtempSync(join(tmpdir(), "docs-freshness-"));

try {
  console.log("Generating to temp directory...");
  console.log(`  Temp: ${tempDir}\n`);

  // Generate all content to temp directory
  const docs = generateDocsPages(tempDir);
  const workflows = generateWorkflowPages(tempDir);
  const reference = generateReferencePages(tempDir);

  const allGenerated = [
    ...docs.generated,
    ...workflows.generated,
    ...reference.generated,
  ];

  console.log(`\nComparing ${allGenerated.length} generated files...\n`);

  const stale = [];
  const missing = [];

  for (const relPath of allGenerated) {
    const tempPath = join(tempDir, relPath);
    const committedPath = join(committedDir, relPath);

    if (!existsSync(committedPath)) {
      missing.push(relPath);
      continue;
    }

    const tempContent = readFileSync(tempPath, "utf-8");
    const committedContent = readFileSync(committedPath, "utf-8");

    if (tempContent !== committedContent) {
      stale.push(relPath);
    }
  }

  if (stale.length === 0 && missing.length === 0) {
    console.log("All generated content is up-to-date.");
    process.exit(0);
  }

  if (stale.length > 0) {
    console.error("STALE files (content differs from source):");
    for (const f of stale) {
      console.error(`  - ${f}`);
    }
  }

  if (missing.length > 0) {
    console.error("MISSING files (not yet generated):");
    for (const f of missing) {
      console.error(`  - ${f}`);
    }
  }

  console.error(`\nRun "node site/scripts/generate-all.mjs" to regenerate.`);
  process.exit(1);
} finally {
  rmSync(tempDir, { recursive: true, force: true });
}
