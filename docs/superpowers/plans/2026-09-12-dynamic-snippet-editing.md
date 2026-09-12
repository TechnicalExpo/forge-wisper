# Dynamic Snippet Editing

**Goal:** Allow users to edit and rename saved voice snippets without changing the existing targeted settings or cleanup architecture.

## Completed

- Existing snippet rows expose an Edit action.
- Edit loads the trigger and expanded text into the existing form.
- Saving can update the snippet value and rename its trigger atomically through the existing `snippets` settings patch.
- Cancel discards edit form changes.
- Existing add, delete, copy, search, pagination, and Rust cleanup preview behavior remain unchanged.
