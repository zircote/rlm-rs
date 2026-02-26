#!/usr/bin/env node

/**
 * Generates Starlight MDX reference pages from GitHub Actions workflow YAML files.
 * Uses simple line-by-line parsing (no external YAML dependency).
 */

import { readFileSync, writeFileSync, mkdirSync, readdirSync } from "node:fs";
import { dirname, join, resolve, basename } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(__dirname, "..", "..");
const siteRoot = resolve(__dirname, "..");

function slugify(name) {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

function parseWorkflowYaml(content) {
  const lines = content.split("\n");
  const name =
    content.match(/^name:\s*['"]?(.+?)['"]?\s*$/m)?.[1]?.trim() || null;

  // Extract triggers from the 'on:' block
  const triggers = [];
  let inOn = false;
  let onIndent = -1;
  for (const line of lines) {
    if (/^on:/.test(line) || /^'on':/.test(line) || /^"on":/.test(line)) {
      inOn = true;
      // Check for inline trigger: on: push
      const inline = line.match(/^(?:on|'on'|"on"):\s+(\w+)/);
      if (inline) {
        triggers.push(inline[1]);
        inOn = false;
      }
      continue;
    }
    if (inOn) {
      // Check if this line has content at a top-level key (no indent) -> end of on: block
      if (/^\S/.test(line) && line.trim() !== "") {
        inOn = false;
        continue;
      }
      // Extract trigger names (indented keys under on:)
      const triggerMatch = line.match(/^\s{2}(\w[\w-]*):/);
      if (triggerMatch) {
        if (onIndent === -1) {
          onIndent = line.search(/\S/);
        }
        if (line.search(/\S/) === onIndent) {
          triggers.push(triggerMatch[1]);
        }
      }
    }
  }

  // Extract job names from 'jobs:' block
  const jobs = [];
  let inJobs = false;
  for (const line of lines) {
    if (/^jobs:/.test(line)) {
      inJobs = true;
      continue;
    }
    if (inJobs) {
      if (/^\S/.test(line) && line.trim() !== "") {
        break;
      }
      const jobMatch = line.match(/^\s{2}([\w-]+):/);
      if (jobMatch && line.search(/\S/) === 2) {
        jobs.push(jobMatch[1]);
      }
    }
  }

  return { name, triggers, jobs };
}

function classifyWorkflow(filename) {
  const base = basename(filename, ".yml");
  if (base.startsWith("ci")) return "CI";
  if (base.startsWith("release")) return "Release";
  if (base.startsWith("publish")) return "Release";
  if (
    base.startsWith("security-") ||
    base.startsWith("codeql") ||
    base.startsWith("secrets-")
  )
    return "Security";
  if (base === "benchmark" || base === "benchmark-regression") return "Testing";
  if (base === "fuzz-testing" || base === "mutation-testing") return "Testing";
  if (base === "nightly") return "Scheduled";
  if (
    ["stale", "dependabot-automerge", "contributors", "spell-check"].includes(
      base,
    )
  )
    return "Maintenance";
  if (
    base === "docs-deploy" ||
    base.startsWith("daily-docs") ||
    base.startsWith("docs-")
  )
    return "Docs";
  if (base.startsWith("package-")) return "Release";
  return "Other";
}

function buildWorkflowPage(filename, parsed, category) {
  const title = parsed.name || basename(filename, ".yml");
  const slug = slugify(basename(filename, ".yml"));
  const description = `Reference for the ${title} GitHub Actions workflow.`;

  let md = `---\ntitle: "${title}"\ndescription: "${description}"\nsidebar:\n  badge:\n    text: "${category}"\n    variant: ${category === "CI" ? '"success"' : category === "Release" ? '"caution"' : category === "Security" ? '"danger"' : '"note"'}\n---\n\n`;

  md += `**Source:** [\`.github/workflows/${filename}\`](https://github.com/zircote/rlm-rs/blob/main/.github/workflows/${filename})\n\n`;

  if (parsed.triggers.length > 0) {
    md += `## Triggers\n\n`;
    md += `| Event |\n|---|\n`;
    for (const t of parsed.triggers) {
      md += `| \`${t}\` |\n`;
    }
    md += "\n";
  }

  if (parsed.jobs.length > 0) {
    md += `## Jobs\n\n`;
    for (const job of parsed.jobs) {
      md += `- \`${job}\`\n`;
    }
    md += "\n";
  }

  return { slug, content: md };
}

/**
 * Generate workflow reference pages.
 * @param {string} [outputBase] - Override output base directory
 * @returns {{ generated: string[] }}
 */
export function generateWorkflowPages(outputBase) {
  const workflowDir = join(projectRoot, ".github", "workflows");
  const outDir = outputBase || join(siteRoot, "src", "content", "docs");
  const generated = [];

  const files = readdirSync(workflowDir).filter((f) => f.endsWith(".yml"));

  for (const file of files) {
    const content = readFileSync(join(workflowDir, file), "utf-8");
    const parsed = parseWorkflowYaml(content);
    const category = classifyWorkflow(file);
    const { slug, content: pageContent } = buildWorkflowPage(
      file,
      parsed,
      category,
    );

    const outPath = join(outDir, "workflows", `ref-${slug}.mdx`);
    mkdirSync(dirname(outPath), { recursive: true });
    writeFileSync(outPath, pageContent, "utf-8");
    console.log(`  OK: workflows/ref-${slug}.mdx [${category}]`);
    generated.push(`workflows/ref-${slug}.mdx`);
  }

  return { generated };
}

// Run directly
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  console.log("Generating workflow reference pages...");
  const { generated } = generateWorkflowPages();
  console.log(`\nDone: ${generated.length} workflow pages generated.`);
}
