# Write release highlights

Add `X.Y.Z-beta.N.md` or `X.Y.Z.md` when preparing a version. Use the exact version from the manifests.
Describe user-visible behavior, migration steps, and known limitations. Omit empty categories and implementation-session history.

The publisher combines these reviewed highlights with GitHub-generated PR categories, contributor attribution, downloads, and a changelog link.
The short highlights also appear in the app updater. Keep them readable as plain text.
Do not edit a published version's highlight file: it is part of that release's signed update-metadata contract.

Use `enhancement`, `bug`, `documentation`, `maintenance`, `dependencies`, and `breaking-change` labels on PRs.
`skip-changelog` omits a PR entry from generated notes; its human authors can still receive contributor credit.
Attribution includes GitHub-linked commit authors and explicit co-author names, without exposing their email addresses.
