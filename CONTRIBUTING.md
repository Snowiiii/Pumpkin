# Contributing to Pumpkin

We appreciate your interest in contributing to Pumpkin! This document outlines the guidelines for submitting bug reports, feature suggestions, and code changes.

## Getting Started

The easiest way to get started is by asking for help in our [discord](https://discord.gg/wT8XjrjKkf).

### How to Contribute

There are several ways you can contribute to Pumpkin:

- **Reporting Bugs**:
  If you encounter a bug, please search for existing issues on the issue tracker first.
  If you can't find a duplicate issue, open a new one.
  Provide a clear description of the bug, including steps to reproduce it if possible.
  Screenshots, logs, or code snippets can also be helpful.
- **Suggesting Features**:
  Do you have an idea on how Pumpkin can be improved? Share your thoughts by opening an issue on the issue tracker.
  Describe the proposed feature in detail, including its benefits and potential implementation considerations.
- **Submitting Pull Requests**:
  If you'd like to contribute code changes, fork the Pumpkin repository on GitHub.
  Install Rust at [rust-lang.org](https://www.rust-lang.org/).
  Make your changes on your local fork and create a pull request to the main repository.
  Ensure your code adheres to our project structure and style guidelines.
  Write clear and concise commit messages that describe your changes.

### Docs

The Documentation of Pumpkin can be found at <https://pumpkinmc.org/>

**Tip: [typos](https://github.com/crate-ci/typos) is a great Project to detect and automatically fix typos**

### Coding Guidelines

Things need to be done before this Pull Request can be merged. Your CI also checks most of them automaticly and fill fail if something is not fulfilled
Note: Pumpkin's clippy settings are relativly strict, this can be may frustrating but is necesarry so the code says clean and conssistent
**Basic**

- **Code Formatting:** Code must be well-formatted and follow the project's style guidelines. You can achieve this by running `cargo fmt`.
- **No Clippy Warnings:** Code should not produce any warnings from the Clippy linter. You can check for warnings using `cargo clippy --all-targets`.
- **Passing Unit Tests:** All existing unit tests must pass successfully. You can run the tests with `cargo test`.

**Best Pratice**

- **Writing Unit Tests:** When adding new features or modifying existing code, consider adding unit tests to prevent regressions in the future. Refer to the Rust documentation for guidance on writing tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
- **Benchmarking:** If your changes might impact performance, consider adding benchmarks to track performance regressions or improvements. We use the Criterion library for benchmarking. Refer to their Quick Start guide for more information: https://github.com/bheisler/criterion.rs#quickstart
- **Clear and Concise Commit Messages:** Use clear and concise commit messages that describe the changes you've made.
- **Code Style:** Adhere to consistent coding style throughout your contributions.
- **Documentation:** If your changes introduce new functionality, consider updating the relevant documentation.
- **Working with Tokio and Rayon:**
  When dealing with CPU-intensive tasks, it's recommended to utilize Rayon's thread pool (`rayon::spawn`), parallel iterators, or similar mechanisms instead of the Tokio runtime. However, it's crucial to avoid blocking the Tokio runtime on Rayon calls. Instead, use asynchronous methods like `tokio::sync::mpsc` to transfer data between the two runtimes. Refer to `pumpkin_world::level::Level::fetch_chunks` for an example of this approach.

### Additional Information

We encourage you to comment on existing issues and pull requests to share your thoughts and provide feedback.
Feel free to ask questions in the issue tracker or reach out to the project maintainers if you need assistance.
Before submitting a large contribution, consider opening an issue, discussion or talk with us on our discord to discuss your approach.
