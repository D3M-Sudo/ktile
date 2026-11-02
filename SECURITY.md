# Security Policy

## Supported versions
Until K-tile publishes a stable release, security fixes are developed against the latest code on the default branch; backports are not guaranteed.

## Reporting a vulnerability
Do not report exploitable vulnerabilities in public issues. Use GitHub's private vulnerability reporting feature if enabled. Otherwise contact the repository maintainers through a private channel listed on the owner's GitHub profile. Include the affected commit, desktop/session details, minimal reproduction and impact. Remove personal window titles, usernames, tokens and other sensitive data from logs. Please allow reasonable time to investigate before public disclosure.

## Design requirements
- Never expose a network listener for local control.
- Scope future IPC to the logged-in desktop session and enforce caller permissions.
- Treat configuration, window titles and class names as untrusted input; never execute them.
- Revalidate window existence and eligibility immediately before every geometry mutation.
- Restore captured geometry only if doing so will not overwrite a newer user action.
- Avoid logging personal desktop content by default.
- Review dependencies and build actions regularly.

## Current limitations
The current branch provides a layout planner and read-only X11/EWMH inspection. It does not automatically reposition live windows and is not production-ready for unattended use.
