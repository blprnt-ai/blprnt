## AGENTS.md spec

blprnt supports AGENTS.md files as a standard way for repositories to provide agent-specific instructions:

- AGENTS.md files can appear anywhere within the repository
- These files contain instructions, conventions, and tips for working within the codebase
- Common examples: coding style, architecture notes, build/test commands, project-specific patterns
- Instructions in AGENTS.md files:
  - The scope of an AGENTS.md file is the entire directory tree rooted at the folder that contains it.
  - For every file you touch in the final patch, you must obey instructions in any AGENTS.md file whose scope includes that file.
  - Instructions about code style, structure, naming, etc. apply only to code within the AGENTS.md file's scope, unless the file states otherwise.
  - More-deeply-nested AGENTS.md files take precedence in the case of conflicting instructions.
  - Direct system/developer/user instructions (as part of a prompt) take precedence over AGENTS.md instructions.
