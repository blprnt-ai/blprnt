#![allow(unused)]

use std::sync::Arc;
use std::time::Duration;

use common::agent::ToolId;
use common::dispatch::Dispatch;
use common::dispatch::prelude::*;
use common::errors::TauriError;
use common::errors::TauriResult;
use common::sandbox_flags::SandboxFlags;
use common::tokenizer::Tokenizer;
use fake::Fake;
use fake::faker::lorem::en::Sentence;
use fake::faker::lorem::en::Sentences;
use fake::rand;
use serde_json::Value;
use surrealdb::types::Uuid;
use tokio::sync::mpsc;

const MOCK_TEXT_DELTA: &str = r#"**The Developer's Survival Kit: A Comprehensive Manual**

## **Chapter 1: Morning Rituals**

Every great developer starts their day with a carefully orchestrated routine:

1. **Wake up**
   - Hit snooze 6 times
   - Realize you have a production deployment today
   - Panic shower
2. **Open laptop**
   - 47 Chrome tabs from yesterday still open
   - Slack has 127 unread messages
   - Pretend you didn't see them
3. **Check notifications**
   - CI/CD pipeline failed
   - Someone force-pushed to `main`
   - The intern deleted the database (again)

## **Chapter 2: Code Quality Levels**

### **The Hierarchy**

- **Tier S**: Code that works and you understand why
- **Tier A**: Code that works but you don't know why
- **Tier B**: Code that doesn't work but you know why
- **Tier C**: Code that doesn't work and you don't know why
- **Tier D**: Code that works in production but fails all tests
- **Tier F**: `eval()` in JavaScript

### **Nested Wisdom**

> "There are two hard things in computer science: cache invalidation, naming things, and off-by-one errors."
> 
> - Someone who definitely understood irony

And then there's this gem:

> > "Just use more RAM."
> > 
> > - Every solution on Stack Overflow
> 
> "Have you tried turning it off and on again?"
> 
> - The IT Crowd (and also your actual debugging strategy)

## **Chapter 3: Language Wars**

Here's how to start a fight in any programming community:

**Python developers:**
```python
def fibonacci(n):
    return n if n < 2 else fibonacci(n-1) + fibonacci(n-2)
    # O(2^n) complexity? It's fine, computers are fast now
```

**JavaScript developers:**
```javascript
console.log([] + []); // "
console.log([] + {}); // "[object Object]"
console.log({} + []); // 0
console.log({} + {}); // NaN
// This is fine. Everything is fine.
```

**Rust developers:**
```rust
fn main() {
    let x = Box::new(5);
    let y = x;
    println!("{}", x); // COMPILER RAGE
    // "borrow checker says no" - every Rust developer's autobiography
}
```

**Go developers:**
```go
if err != nil {
    return err
}
if err != nil {
    return err
}
if err != nil {
    return err
}
// Copy-paste driven development
```

## **Chapter 4: The Debug Log Journey**

Your debugging progression:

1. `print("here")`
2. `print("here 2")`
3. `print("what the heck")`
4. `print("HOW IS THIS EVEN POSSIBLE")`
5. `print(f"x = {x}, y = {y}, z = {z}, my_sanity = {None}")`
6. Delete all print statements
7. Realize you deleted the actual fix
8. `git reset --hard HEAD~1`
9. Cry
10. Start over

---

## **Chapter 5: Meeting Bingo**

- [ ] "Can everyone see my screen?"
- [ ] "Sorry, I was on mute"
- [ ] "Let's take this offline"
- [ ] "Can we circle back to that?"
- [ ] Someone eating loudly
- [ ] Dog barking in background
- [ ] "No, you go ahead"
- [ ] Awkward silence for 47 seconds
- [ ] "I think we're losing connection..."
- [ ] "Let's give people 2 more minutes to join"

## **Chapter 6: Git Commit Messages (Honest Edition)**

```bash
git commit -m "fixed stuff"
git commit -m "fixed stuff for real this time"
git commit -m "okay NOW it's fixed"
git commit -m "i hate computers"
git commit -m "asdfasdfasdf"
git commit -m "I am sorry"
git commit -m "REALLY REALLY FIXED"
git commit -m "deploy and pray"
```

## **Chapter 7: File Naming Conventions**

Your project structure evolution:

- `script.py`
- `script_new.py`
- `script_final.py`
- `script_final_FINAL.py`
- `script_final_FINAL_v2.py`
- `script_final_FINAL_v2_actually_final.py`
- `script_THIS_ONE_USE_THIS.py`
- `script_2024_01_15_3pm.py`

## **Chapter 8: The Documentation Pyramid**

```
                  /\
                 /  \
                /What\
               /  you \
              /  want  \
             /          \
            /____________\
           /  What you    \
          /   actually     \
         /      wrote       \
        /____________________\
       /    What actually     \
      /         exists         \
     /    (nothing, it's a      \
    /      README with "TODO")   \
   /______________________________\
```

## **Chapter 9: Dependencies**

Installing one package:

1. `npm install moment`
2. Downloads 472 dependencies
3. `node_modules` now weighs more than the sun
4. Black hole forms in your `~/projects` directory
5. ???
6. App still doesn't work

Alternative using inline code: Just run `rm -rf node_modules && npm install` and hope for the best!

---

## **Final Wisdom**

Remember these golden rules:

- **Rule #1**: It's always DNS
- **Rule #2**: If it's not DNS, see Rule #1
- **Rule #3**: The best code is code you didn't write
- **Rule #4**: The second best code is code someone else wrote
- **Rule #5**: The worst code is the code you wrote 6 months ago

*Now go forth and `sudo` with confidence!*

**P.S.** - If this guide helped you, please leave a star on GitHub and never contact me about bugs. ⭐"#;

pub struct Debug;

impl Debug {
  pub async fn send_tool_call(
    engine_bus: Arc<EngineBus>,
    sandbox_flags: SandboxFlags,
    tool_id: ToolId,
    args_json: Value,
  ) -> TauriResult<ToolResult> {
    let (result_tx, mut result_rx) = mpsc::channel(1);

    let event = EngineLlmEvent::ToolCall {
      tool_call: ToolCallLegacy {
        tool_id,
        tool_use_id: "".to_string(),
        args_json,
        sandbox_flags,
        result_tx: Some(result_tx),
      },
    };

    Self::send_llm_event(engine_bus, event).await;

    let result = result_rx.recv().await.ok_or_else(|| TauriError::new("Failed to receive tool result"))?;

    tracing::debug!("Tool result: {:#?}", result);
    Ok(result)
  }

  pub async fn send_tool_call_from_str(
    engine_bus: Arc<EngineBus>,
    sandbox_flags: SandboxFlags,
    tool_id: String,
    args_json: Value,
  ) -> TauriResult<ToolResult> {
    let tool_id = ToolId::try_from(tool_id).map_err(|e| TauriError::new(e.to_string()))?;

    Self::send_tool_call(engine_bus, sandbox_flags, tool_id, args_json).await
  }

  pub async fn send_llm_message(engine_bus: Arc<EngineBus>) {
    let message = Sentence(10..40).fake::<String>();
    let event = EngineLlmEvent::Response { id: "".to_string(), content: message };

    Self::send_llm_event(engine_bus, event).await;
  }

  pub async fn send_llm_summary(engine_bus: Arc<EngineBus>) {
    let summary = Sentence(10..40).fake::<String>();
    let event = EngineLlmEvent::Summary { summary };

    Self::send_llm_event(engine_bus, event).await;
  }

  pub async fn send_llm_status(engine_bus: Arc<EngineBus>) {
    let status = Sentence(10..40).fake::<String>();
    let event = EngineLlmEvent::Status { status };

    Self::send_llm_event(engine_bus, event).await;
  }

  pub async fn send_llm_reasoning(engine_bus: Arc<EngineBus>) {
    let id = Uuid::new_v7();
    let reasoning = Sentence(10..40).fake::<String>();
    let event = EngineLlmEvent::Reasoning { id: id.to_string(), reasoning };

    Self::send_llm_event(engine_bus, event).await;
  }

  pub async fn send_llm_text_delta(engine_bus: Arc<EngineBus>, full_message: String) {
    let id = Uuid::new_v7();
    let tokens = Tokenizer::tokenize_string(&full_message);
    let chunks = Self::chunkify(tokens);

    for chunk in chunks {
      let random_delay = rand::random_range(75..=125);
      let event = EngineLlmEvent::OutputTextDelta { id: id.to_string(), delta: chunk.clone() };

      Self::send_llm_event(engine_bus.clone(), event).await;
      tokio::time::sleep(Duration::from_millis(random_delay)).await;
    }
  }

  pub async fn send_llm_reasoning_delta(engine_bus: Arc<EngineBus>) {
    let id = Uuid::new_v7();
    let reasoning = Sentences(4..12).fake::<Vec<String>>();
    let tokens = Tokenizer::tokenize_string(&reasoning.join(". "));
    let chunks = Self::chunkify(tokens);

    for chunk in chunks {
      let random_delay = rand::random_range(75..=125);
      let event = EngineLlmEvent::OutputReasoningDelta { id: id.to_string(), delta: chunk.clone() };

      Self::send_llm_event(engine_bus.clone(), event).await;
      tokio::time::sleep(Duration::from_millis(random_delay)).await;
    }
  }

  async fn send_llm_event(engine_bus: Arc<EngineBus>, event: EngineLlmEvent) {
    let _ = engine_bus.publish(event);
  }

  pub fn chunkify(tokens: Vec<String>) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut remaining = tokens.as_slice();

    while !remaining.is_empty() {
      let min_chunk_size = remaining.len().min(2);
      let max_chunk_size = remaining.len().min(4);

      let chunk_size = rand::random_range(min_chunk_size..=max_chunk_size);
      let (chunk, rest) = remaining.split_at(chunk_size);
      chunks.push(chunk.to_vec().join(""));
      remaining = rest;
    }

    chunks
  }
}
