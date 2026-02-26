#!/usr/bin/env node

/**
 * Post-build script that embeds rustdoc output into the Starlight site.
 * If target/doc doesn't exist, runs cargo doc first.
 * Copies rustdoc output to site/dist/api/ and creates a redirect index.
 */

import { existsSync, cpSync, mkdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(__dirname, "..", "..");
const siteRoot = resolve(__dirname, "..");
const docSource = join(projectRoot, "target", "doc");
const docDest = join(siteRoot, "dist", "api");

// Build rustdoc if not already present
if (!existsSync(docSource)) {
  console.log("target/doc not found, running cargo doc...");
  execFileSync("cargo", ["doc", "--no-deps", "--all-features"], {
    cwd: projectRoot,
    stdio: "inherit",
  });
}

if (!existsSync(docSource)) {
  console.error("ERROR: cargo doc did not produce target/doc/");
  process.exit(1);
}

// Copy rustdoc output to dist/api/
console.log(`Copying ${docSource} -> ${docDest}`);
mkdirSync(docDest, { recursive: true });
cpSync(docSource, docDest, { recursive: true });

// Create redirect index.html -> rlm_rs/
const redirectHtml = `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta http-equiv="refresh" content="0;url=rlm_rs/">
  <title>API Documentation</title>
</head>
<body>
  <p>Redirecting to <a href="rlm_rs/">rlm_rs API docs</a>...</p>
</body>
</html>
`;

const indexPath = join(docDest, "index.html");
if (!existsSync(indexPath)) {
  writeFileSync(indexPath, redirectHtml, "utf-8");
  console.log("Created redirect index.html -> rlm_rs/");
}

console.log("Rustdoc embedding complete.");
