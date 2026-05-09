# Interface Specification: tachyon:ai/viking-context

This WIT definition standardizes how the Pulsar agent fetches hierarchical codebase memory.

```wit
package tachyon:ai@1.0.0;

interface viking-context {
    /// Depth of the semantic context required by the agent
    enum context-level {
        l0-summary,
        l1-structure,
        l2-raw,
    }

    /// Represents a resolved chunk of context ready for LLM prompt injection
    record context-response {
        /// The URI that was resolved
        uri: string,
        /// The actual level resolved (may differ if fallback occurred)
        level: context-level,
        /// The text payload
        payload: string,
        /// Estimated token count to help the agent manage its context window
        token-estimate: u32,
    }

    /// Resolves a file into a structured, token-efficient payload
    resolve: func(uri: string, level: context-level) -> result<context-response, string>;
}
```