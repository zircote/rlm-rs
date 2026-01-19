---
# Image generation provider
# Options: svg, dalle-3, gemini, manual
provider: svg

# SVG-specific settings (only used when provider: svg)
# Options: minimal, geometric, illustrated
svg_style: minimal

# Dark mode support
# false = light mode only, true = dark mode only, both = generate both variants
dark_mode: false

# Output settings
output_path: .github/social-preview.svg
dimensions: 1280x640
include_text: true
colors: auto

# README infographic settings
infographic_output: .github/readme-infographic.svg
infographic_style: hybrid

# Upload to repository (requires gh CLI or GITHUB_TOKEN)
upload_to_repo: false
---

# GitHub Social Plugin Configuration

This configuration was created by `/github-social:setup`.

## Provider: SVG (Recommended)

Claude generates clean, minimal SVG graphics directly. No API key required.

**Benefits:**
- Free - no API costs
- Instant - no network calls
- Editable - can be modified in any vector editor
- Small file size (10-50KB)
- Professional, predictable results

## Style: Minimal

Clean, simple design with project name and subtle geometric accents:
- Maximum 3-5 shapes
- Generous whitespace
- Professional appearance
- Focus on typography

## Commands

Generate social preview:
```bash
/social-preview
```

Enhance README with badges and infographic:
```bash
/readme-enhance
```

Run all github-social skills:
```bash
/github-social:all
```

## Override Settings

Override any setting via command flags:
```bash
/social-preview --provider=dalle-3 --dark-mode
/social-preview --svg-style=geometric
```

## Modify Configuration

Edit this file or run `/github-social:setup` again to reconfigure.
