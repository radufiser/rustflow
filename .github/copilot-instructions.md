---
applyTo: '**'
---

---
inclusion: always
---

# Shell Command Guidelines

CRITICAL: Do not use interactive commands or pagers (e.g., less, more, vim). All shell commands must be non-interactive.
Ensure you use flags like -y for confirmations and --no-pager for CLI tools.
If a command typically opens a pager, pipe it to cat or set PAGER=cat in the environment to ensure the full output
is returned without waiting for user input.