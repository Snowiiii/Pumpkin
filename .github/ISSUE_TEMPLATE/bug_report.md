---
name: Bug report
about: Create a bug report to help us improve
title: ""
labels: bug
assignees: Snowiiii
body:
- type: checkboxes
  attributes:
    label: I've searched existing issues and couldn't find a duplicate.
    description: I confirm this is not a duplicate.
  validations:
    required: true
- type: textarea
  attributes:
    label: Operating System
    description: What operating system are you using?
    placeholder: "Example: macOS Big Sur"
    value: operating system
  validations:
    required: true
- type: textarea
  attributes:
    label: Server Software Version/Commit
    description: What Server Software Version/Commit are you using?
    placeholder: "Example: 1.0.0/39b4cb3"
    value: version/commit
  validations:
    required: true
---

**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:

**Expected behavior**
A clear and concise description of what you expected to happen.

**Additional context**
Add any other context about the problem here.
