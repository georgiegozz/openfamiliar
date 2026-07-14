# Governance

## Project stage

OpenFamiliar is currently a **single-maintainer private project** preparing for
a public open-source beta.

## Decision making

| Area                       | Owner                                    |
| -------------------------- | ---------------------------------------- |
| Product scope & roadmap    | Maintainer (georgiegozz)                 |
| Architecture ADRs          | Maintainer; community review once public |
| Security-sensitive changes | Maintainer approval required             |
| License / trademark policy | Maintainer                               |

## ADRs

Significant architectural decisions are recorded under `docs/adr/` and should
be accepted before large implementations land on `main`.

## Releases

- Semantic Versioning (SemVer)
- First public tag target: `v0.1.0` Public Beta (not v1.0.0)
- v1.0.0 reserved for stable `.familiar` schema, documented SDK compatibility,
  deprecation policy, and at least two stable providers + two agent adapters

## Contributions

See `CONTRIBUTING.md`. All contributions require DCO sign-off.

## Future

When the community grows, this document may introduce:

- CODEOWNERS
- Security team
- Release managers
- RFC process for large features
