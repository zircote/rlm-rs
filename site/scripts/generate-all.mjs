#!/usr/bin/env node

/**
 * Master generation script. Runs all content generators.
 */

import { existsSync, copyFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { generateDocsPages } from "./generate-docs-pages.mjs";
import { generateWorkflowPages } from "./generate-workflow-pages.mjs";
import { generateReferencePages } from "./generate-reference-pages.mjs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(__dirname, "..", "..");
const siteRoot = resolve(__dirname, "..");

// Use .github/social-preview.svg as OG image if available
const socialPreview = join(projectRoot, ".github", "social-preview.svg");
const ogImage = join(siteRoot, "public", "og-image.svg");
if (existsSync(socialPreview)) {
  copyFileSync(socialPreview, ogImage);
  console.log(
    "Copied .github/social-preview.svg -> site/public/og-image.svg\n",
  );
}

console.log("=== Generating docs pages ===");
const docs = generateDocsPages();

console.log("\n=== Generating workflow pages ===");
const workflows = generateWorkflowPages();

console.log("\n=== Generating reference pages ===");
const reference = generateReferencePages();

const total =
  docs.generated.length +
  workflows.generated.length +
  reference.generated.length;
console.log(`\nAll generation complete. ${total} pages generated.`);
