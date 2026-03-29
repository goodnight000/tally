# Testing

100% test coverage is the key to great vibe coding. Tests let you move fast, trust your instincts, and ship with confidence. Without them, vibe coding is just yolo coding. With tests, it's a superpower.

## Framework

- **Vitest** v4.x with jsdom environment
- **@testing-library/react** for component tests
- **@testing-library/jest-dom** for DOM assertions

## Running Tests

```bash
# Run all tests once
npm test

# Watch mode (re-runs on change)
npm run test:watch

# Run a specific test file
npx vitest run src/lib/format.test.ts
```

## Test Layers

- **Unit tests** — Pure functions in `src/lib/`. Test formatting, calculations, date logic. Located next to source files as `*.test.ts`.
- **Component tests** — React components in `src/components/` and `src/pages/`. Test rendering, interactions, state. Located next to source files as `*.test.tsx`.
- **Integration tests** — Multi-component flows. Mock Tauri `invoke()` calls, test full page behavior.

## Conventions

- Test files live next to their source: `format.ts` → `format.test.ts`
- Use `describe` blocks grouped by function/component name
- Use plain `it`/`expect` assertions, no custom matchers
- Mock Tauri IPC calls with `vi.mock("@tauri-apps/api/core")`
