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

### Faster Compiles

We use extism for our plugins, unfortunately extism has quite a hefty impact on compile times, which can increase it by several minutes on release mode. To run Pumpkin without plugins, run the following:

```shell
cargo run --no-default-features
```

And in release:

```shell
cargo run --no-default-features --release
```

### Docs
The Documentation of Pumpkin can be found at https://snowiiii.github.io/Pumpkin/

### Additional Information

We encourage you to comment on existing issues and pull requests to share your thoughts and provide feedback.
Feel free to ask questions in the issue tracker or reach out to the project maintainers if you need assistance.
Before submitting a large contribution, consider opening an issue, discussion or talk with us on our discord to discuss your approach.
