---
name: Documentation Site Freshness Check
description: |
  Weekly check that generated Starlight documentation site content is in sync with source files.
  Creates or updates a GitHub issue when content is stale.
on:
  schedule:
    - cron: 'weekly on sunday'
  workflow_dispatch:

permissions:
  contents: read
  issues: read

engine:
  id: copilot

timeout-minutes: 15

tools:
  bash:
    - echo
    - cat
    - grep
    - node
    - npm
    - npx
  github:
    toolsets: [context, issues]

safe-outputs:
  create-issue:
    title-prefix: "[docs-freshness] "
    close-older-issues: true
  add-comment:
    target: "*"
    discussions: false
    max: 1
---

# Documentation Site Freshness Check

## Context

This repository has an Astro Starlight documentation site in `site/` that generates MDX content pages from three sources:

1. **Docs pages** — markdown files in `docs/` are converted to Starlight MDX pages via `site/scripts/generate-docs-pages.mjs`
2. **Workflow reference pages** — `.github/workflows/*.yml` files are parsed into reference pages via `site/scripts/generate-workflow-pages.mjs`
3. **Reference pages** — sections from `CLAUDE.md` are extracted into individual reference pages via `site/scripts/generate-reference-pages.mjs`

The generated content lives in `site/src/content/docs/` and must stay in sync with these sources.

## Instructions

1. **Install dependencies**: Run `cd site && npm ci` to install the site's Node.js dependencies.

2. **Run the freshness check**: Execute `npm run check:freshness` in the `site/` directory.
   - This script generates content to a temp directory and diffs it against the committed content.
   - Exit code 0 means all content is up-to-date.
   - Exit code 1 means content is stale, and the script lists which files need regeneration.

3. **If content is up-to-date** (exit code 0):
   - Output a brief summary: "All generated documentation pages are up-to-date."
   - Do not create any issues.

4. **If content is stale** (exit code 1):
   - Capture the list of stale files from the script output.
   - Search for an existing open issue with the title prefix `[docs-freshness]`.
   - **If an open issue exists**: Add a comment to it with the updated list of stale files and the current date.
   - **If no open issue exists**: Create a new issue titled `[docs-freshness] Documentation site content is stale` with:
     - The list of stale/missing files
     - Instructions to fix: `cd site && npm run generate && git add . && git commit -m "docs: regenerate site content"`
     - A note that this was detected by the automated freshness check

## Edge Cases

- If `npm ci` fails, report the error and create an issue about broken site dependencies.
- If the freshness script itself errors (not exit code 0 or 1), report the unexpected error.
