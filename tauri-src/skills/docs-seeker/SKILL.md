---
name: docs-seeker
description: "Searching internet for technical documentation using llms.txt standard and web search exploration. Use when user needs: (1) Latest documentation for libraries/frameworks, (2) Documentation in llms.txt format, (3) GitHub repository analysis, (4) Documentation without direct llms.txt support"
version: 1.0.0
---

# Documentation Discovery & Analysis

## Overview

Intelligent discovery and analysis of technical documentation through multiple strategies:

1. **llms.txt-first**: Search for standardized AI-friendly documentation
2. **Fallback research**: Use web search when other methods unavailable

## Core Workflow

### Phase 1: Initial Discovery

1. **Identify target**
   - Extract library/framework name from user request
   - Note version requirements (default: latest)
   - Clarify scope if ambiguous
   - Identify if target is GitHub repository or website

2. **Search for llms.txt (PRIORITIZE context7.com)**

   **First: Try context7.com patterns**

   For GitHub repositories:
   ```
   Pattern: https://context7.com/{org}/{repo}/llms.txt
   Examples:
   - https://github.com/imagick/imagick → https://context7.com/imagick/imagick/llms.txt
   - https://github.com/vercel/next.js → https://context7.com/vercel/next.js/llms.txt
   - https://github.com/better-auth/better-auth → https://context7.com/better-auth/better-auth/llms.txt
   ```

   For websites:
   ```
   Pattern: https://context7.com/websites/{normalized-domain-path}/llms.txt
   Examples:
   - https://docs.imgix.com/ → https://context7.com/websites/imgix/llms.txt
   - https://docs.byteplus.com/en/docs/ModelArk/ → https://context7.com/websites/byteplus_en_modelark/llms.txt
   - https://docs.haystack.deepset.ai/docs → https://context7.com/websites/haystack_deepset_ai/llms.txt
   - https://ffmpeg.org/doxygen/8.0/ → https://context7.com/websites/ffmpeg_doxygen_8_0/llms.txt
   ```

   **Topic-specific searches** (when user asks about specific feature):
   ```
   Pattern: https://context7.com/{path}/llms.txt?topic={query}
   Examples:
   - https://context7.com/shadcn-ui/ui/llms.txt?topic=date
   - https://context7.com/shadcn-ui/ui/llms.txt?topic=button
   - https://context7.com/vercel/next.js/llms.txt?topic=cache
   - https://context7.com/websites/ffmpeg_doxygen_8_0/llms.txt?topic=compress
   ```

   **Fallback: Traditional llms.txt search**
   ```
   WebSearch: "[library name] llms.txt site:[docs domain]"
   ```
   Common patterns:
   - `https://docs.[library].com/llms.txt`
   - `https://[library].dev/llms.txt`
   - `https://[library].io/llms.txt`

   → Found? Proceed to Phase 2
   → Not found? Proceed to Phase 3

### Phase 2: llms.txt Processing

**Single URL:**
- WebSeach to retrieve content
- Extract and present information

### Phase 3: Fallback Research

**When no GitHub repository exists:**
- Focus areas: official docs, tutorials, API references, community guides
- Aggregate findings into consolidated report

## Version Handling

**Latest (default):**
- Search without version specifier
- Use current documentation paths

**Specific version:**
- Include version in search: `[library] v[version] llms.txt`
- Check versioned paths: `/v[version]/llms.txt`
- For repositories: checkout specific tag/branch

## Output Format

```markdown
# Documentation for [Library] [Version]

## Source
- Method: [llms.txt / Repository / Research]
- URLs: [list of sources]
- Date accessed: [current date]

## Key Information
[Extracted relevant information organized by topic]

## Additional Resources
[Related links, examples, references]

## Notes
[Any limitations, missing information, or caveats]
```


**Popular llms.txt locations (try context7.com first):**
- Astro: https://context7.com/withastro/astro/llms.txt
- Next.js: https://context7.com/vercel/next.js/llms.txt
- Remix: https://context7.com/remix-run/remix/llms.txt
- shadcn/ui: https://context7.com/shadcn-ui/ui/llms.txt
- Better Auth: https://context7.com/better-auth/better-auth/llms.txt

**Fallback to official sites if context7.com unavailable:**
- Astro: https://docs.astro.build/llms.txt
- Next.js: https://nextjs.org/llms.txt
- Remix: https://remix.run/llms.txt
- SvelteKit: https://kit.svelte.dev/llms.txt

## Error Handling

- **llms.txt not accessible** → Try alternative domains → Repository analysis
- **Repository not found** → Search official website
- **Multiple conflicting sources** → Prioritize official → Note versions

## Key Principles

1. **Prioritize context7.com for llms.txt** — Most comprehensive and up-to-date aggregator
2. **Use topic parameters when applicable** — Enables targeted searches with ?topic=...
3. **Verify official sources as fallback** — Use when context7.com unavailable
4. **Report methodology** — Tell user which approach was used
5. **Handle versions explicitly** — Don't assume latest

## Detailed Documentation

For comprehensive guides, examples, and best practices:

**Workflows:**
- [docs-seeker/WORKFLOWS.md](docs-seeker/WORKFLOWS.md) — Detailed workflow examples and strategies

**Reference guides:**
- [references/tool-selection.md](references/tool-selection.md) — Complete guide to choosing and using tools
- [references/documentation-sources.md](references/documentation-sources.md) — Common sources and patterns across ecosystems
- [references/error-handling.md](references/error-handling.md) — Troubleshooting and resolution strategies
- [references/best-practices.md](references/best-practices.md) — 8 essential principles for effective discovery
- [references/performance.md](references/performance.md) — Optimization techniques and benchmarks
- [references/limitations.md](references/limitations.md) — Boundaries and success criteria
