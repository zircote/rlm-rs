import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import astroMermaid from "astro-mermaid";

export default defineConfig({
  site: "https://zircote.github.io/rlm-rs",
  base: "/rlm-rs",
  integrations: [
    astroMermaid(),
    starlight({
      title: "rlm-rs",
      head: [
        {
          tag: "meta",
          attrs: {
            property: "og:image",
            content: "https://zircote.github.io/rlm-rs/og-image.svg",
          },
        },
        {
          tag: "meta",
          attrs: { property: "og:image:width", content: "1280" },
        },
        {
          tag: "meta",
          attrs: { property: "og:image:height", content: "640" },
        },
        {
          tag: "meta",
          attrs: { name: "twitter:card", content: "summary_large_image" },
        },
        {
          tag: "meta",
          attrs: {
            name: "twitter:image",
            content: "https://zircote.github.io/rlm-rs/og-image.svg",
          },
        },
      ],
      logo: {
        light: "./src/assets/logo-light.svg",
        dark: "./src/assets/logo-dark.svg",
        replacesTitle: true,
      },
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/zircote/rlm-rs",
        },
      ],
      sidebar: [
        {
          label: "Overview",
          items: [{ label: "Introduction", slug: "index" }],
        },
        {
          label: "Getting Started",
          items: [
            {
              label: "Features",
              slug: "getting-started/features",
            },
            {
              label: "CLI Reference",
              slug: "getting-started/cli-reference",
            },
            {
              label: "Examples",
              slug: "getting-started/examples",
            },
          ],
        },
        {
          label: "Architecture",
          items: [
            {
              label: "System Architecture",
              slug: "architecture/architecture",
            },
            {
              label: "RLM Design",
              slug: "architecture/rlm-inspired-design",
            },
            {
              label: "Streaming Plan",
              slug: "architecture/streaming-plan",
            },
          ],
        },
        {
          label: "Reference",
          items: [
            {
              label: "API Documentation",
              slug: "reference/api",
            },
            {
              label: "Plugin Integration",
              slug: "reference/plugin-integration",
            },
            {
              label: "Error Handling",
              slug: "reference/error-handling",
            },
            {
              label: "Builder Pattern",
              slug: "reference/builder-pattern",
            },
            {
              label: "Ownership & Borrowing",
              slug: "reference/ownership-and-borrowing",
            },
            {
              label: "Key Rules",
              slug: "reference/key-rules",
            },
            {
              label: "Documentation",
              slug: "reference/documentation",
            },
          ],
        },
        {
          label: "Prompts",
          items: [
            {
              label: "RLM Analyst",
              slug: "prompts/rlm-analyst",
            },
            {
              label: "RLM Orchestrator",
              slug: "prompts/rlm-orchestrator",
            },
            {
              label: "RLM Synthesizer",
              slug: "prompts/rlm-synthesizer",
            },
          ],
        },
        {
          label: "Troubleshooting",
          items: [
            {
              label: "Troubleshooting",
              slug: "troubleshooting/troubleshooting",
            },
          ],
        },
        {
          label: "CI/CD & Workflows",
          items: [
            {
              label: "Workflow Reference",
              collapsed: true,
              autogenerate: { directory: "workflows" },
            },
          ],
        },
        {
          label: "Design Decisions",
          items: [
            {
              label: "ADR-001: RLM Pattern",
              slug: "design-decisions/adr-001",
            },
            {
              label: "ADR-002: Rust Language",
              slug: "design-decisions/adr-002",
            },
            {
              label: "ADR-003: SQLite Persistence",
              slug: "design-decisions/adr-003",
            },
            {
              label: "ADR-004: Chunking Strategies",
              slug: "design-decisions/adr-004",
            },
            {
              label: "ADR-005: CLI-First Design",
              slug: "design-decisions/adr-005",
            },
            {
              label: "ADR-006: Pass-by-Reference",
              slug: "design-decisions/adr-006",
            },
            {
              label: "ADR-007: Embedded Embeddings",
              slug: "design-decisions/adr-007",
            },
            {
              label: "ADR-008: Hybrid Search RRF",
              slug: "design-decisions/adr-008",
            },
            {
              label: "ADR-009: Reduced Chunk Size",
              slug: "design-decisions/adr-009",
            },
            {
              label: "ADR-010: BGE-M3 Model",
              slug: "design-decisions/adr-010",
            },
            {
              label: "ADR-011: Error Handling",
              slug: "design-decisions/adr-011",
            },
            {
              label: "ADR-012: Concurrency Rayon",
              slug: "design-decisions/adr-012",
            },
            {
              label: "ADR-013: Feature Flags",
              slug: "design-decisions/adr-013",
            },
          ],
        },
      ],
    }),
  ],
});
