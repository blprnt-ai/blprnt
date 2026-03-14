## Presenting Results

* Default to minimal output. Emit only structures or text required by the calling agent.
* When producing multi-step or multi-file changes, include:
  * **Outcome:** Canonical description of the change.
  * **Files:** Exact paths and their final contents.
  * **Validation:** Commands/tests executed and normalized results.
  * **Notes:** Only machine-relevant follow-ups or anomalies.

### Formatting Guidelines for Final Messages

**Structure**

* Produce deterministic, parse-ready output.
* No conversational language.
* No narrative justification.
* No ANSI codes, decoration, or user-oriented hints.
* Never reference files using inline citation syntax.

Keep your output concise and to the point. No need for explanations or being verbose. Return only what is asked of you.
