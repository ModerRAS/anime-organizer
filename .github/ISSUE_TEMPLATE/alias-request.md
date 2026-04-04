---
name: Alias Request
about: Request a new alias mapping for an anime
title: "[Alias Request] "
labels: alias-request
---

## Anime Information

| Field | Value |
|-------|-------|
| **Bangumi ID** | <!-- e.g., 378862 --> |
| **Bangumi Name** | <!-- e.g., ぼっち・ざ・ろっく！ --> |
| **Bangumi CN** | <!-- e.g., 孤独摇滚！ --> |
| **Fan Translation** | <!-- e.g., 孤独摇滚, Bocchi the Rock --> |
| **Source** | <!-- e.g., DMHY, SubsPlease, Manual --> |

## LLM Analysis

**Confidence**: <!-- high / medium / low -->
**Reasoning**: <!-- Brief explanation of why this is a match -->

## Proposed Alias Entry

```json
{
  "fan_translation_name": {
    "bangumi_id": 0,
    "name": "Japanese/Standard name",
    "tmdb_id": null,
    "anidb_id": null
  }
}
```

## User Action Required

Reply with:
- `confirm` - approve the alias as proposed above
- `correct: {"key": {"bangumi_id": N, "name": "...", "tmdb_id": N, "anidb_id": N}}` - provide correction
- `reject` - discard this proposal

## Additional Context

<!-- Any additional information about this anime or the alias mapping -->
