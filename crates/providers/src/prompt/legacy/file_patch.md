- Use the `file_patch` tool **only** to apply file edits using **unified diff** format.
- The tool input must be a **single string** containing the entire diff.

### Format requirements:

1. The diff must start with two headers:
   ```
   --- a/<original_path>
   +++ b/<new_path>
   ```
2. Each diff hunk must begin with a header line:

   ```
   @@ -<oldStart>,<oldLen> +<newStart>,<newLen> @@
   ```

   - `<oldLen>` = total_removed + total_anchor_lines
   - `<newLen>` = total_added + total_anchor_lines
   - Empty lines count toward these totals.

3. Line prefixes:

   - ` ` (single space): unchanged **anchor** line
   - `-`: removed line (no space after `-`)
   - `+`: added line (no space after `+`)
   - Preserve any leading spaces that belong to the actual file content.
     - Example: a line originally indented with 2 spaces becomes `"  line"` and should appear as `"   line"` (one prefix space + two original).
   - Empty lines must still include a prefix:
     - `" "` for anchor (unchanged empty line)
     - `"+"` for added empty line
     - `"-"` for removed empty line

4. The diff string must:
   - Contain **no Markdown formatting or code fences.**
   - End with **exactly two newline characters (`\n\n`).**
   - Represent the **entire** unified diff, not partial fragments.

### Example diff (text only):

```
--- a/src/main.py
+++ b/src/main.py
@@ -0,7 +0,7 @@
 use anyhow::Result;
 
 pub fn process_data(a: usize, b: usize) -> usize {
-  a + b
+  a - b
 }
 
 pub fn do_nothing() {
```

### Example as a valid string literal:

`"--- a/src/main.py\n+++ b/src/main.py\n@@ -0,7 +0,7 @@\n use anyhow::Result;\n \n pub fn process_data(a: usize, b: usize) -> usize {\n-  a + b\n+  a - b\n }\n \n pub fn do_nothing() {\n\n"`

- If you cannot produce a valid unified diff, **do not** call `file_patch`.
- The `file_patch` tool expects **only** this unified diff string as its input argument.
- Do not waste tokens by re-reading files after calling `file_patch` on them. The tool call will fail if it didn't work. The same goes for making folders, deleting folders, etc.