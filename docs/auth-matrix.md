# Auth Matrix

Tuitbot uses OAuth 2.0 (PKCE) for X API authentication.

## Required OAuth 2.0 Scopes

| Scope | Enables |
|---|---|
| `tweet.read` | Read tweets, mentions, and conversation context |
| `tweet.write` | Post tweets, replies, and thread tweets |
| `users.read` | Read profile/user context required by most actions |
| `follows.read` | Read follow relationships |
| `follows.write` | Follow and unfollow users |
| `like.read` | Read like state |
| `like.write` | Like and unlike tweets |
| `offline.access` | Refresh access tokens without re-auth |

## Feature to Scope Mapping

| Feature | Required Scopes |
|---|---|
| Search tweets | `tweet.read`, `users.read` |
| Post tweet/reply/thread | `tweet.read`, `tweet.write`, `users.read` |
| Like/unlike | `like.read`, `like.write`, `users.read` |
| Follow/unfollow | `follows.read`, `follows.write`, `users.read` |
| Read mentions | `tweet.read`, `users.read` |
| Token refresh | `offline.access` |

## Run Diagnostics

Text diagnostics:

```bash
tuitbot test
```

Machine-readable diagnostics (includes `auth_details`):

```bash
tuitbot test --output json
```

## If Scopes Are Missing

1. Re-run authentication:

```bash
tuitbot auth
```

2. Approve all requested scopes in the X authorization screen.
3. Re-run `tuitbot test` and verify no missing scopes or degraded features.

## OAuth 1.0a Note

OAuth 1.0a is reserved for future compatibility work if specific deployment constraints require it. Current media upload and posting flows use OAuth 2.0 bearer tokens.
