# Contributing Guidelines

We welcome and appreciate contributions to the WhatsUp WhatsApp Broadcast platform! To ensure a smooth process, please follow the guidelines documented below.

## 🛠️ Development Principles
1.  **Strict Type Safety**: All TypeScript code must compile successfully without warnings. Avoid using `any` type definitions unless strictly necessary; prefer descriptive custom interfaces.
2.  **Lint Check Validation**: Run lints and formatting validations before committing code. We enforce standard Cargo fmt for the Rust API and ESLint + Prettier rules for Next.js.
3.  **Rust Safety**: Keep code free of `unwrap()` statements inside handlers; utilize standard error propagation pattern (`?`) and proper error mapping wrapper structs.

---

## 🍴 Pull Request Workflow

### Step 1: Branch Strategy
Choose a branch name reflecting your task category:
*   `feature/your-feature-name` (for additions)
*   `bugfix/issue-description` (for patches)
*   `docs/doc-updates` (for edits to documentation files)

### Step 2: Make Changes locally
Make sure your changes are clean.
*   **Rust (Backend)**: Verify locally using `cargo clippy` and `cargo test`.
*   **Next.js (Frontend)**: Run `bun run type-check` and `bun run lint`.

### Step 3: Open Pull Request
*   Target the `main` branch.
*   Describe your changes clearly in the PR description template.
*   Ensure that the GitHub Actions CI builds compile successfully before requesting code reviews.

---

## 💬 Code of Conduct
Please review our [Code of Conduct](CODE_OF_CONDUCT.md) before collaborating. Let's make this community welcoming, inclusive, and professional!
