# Admin dashboard access fix

## Root cause found

The backend reads `users.user_role` from PostgreSQL on every authenticated request, so a direct DB update like:

```sql
UPDATE users SET user_role = 'admin' WHERE email = 'you@example.com';
```

is enough for backend authorization **if the current session is valid**.

The broken part was the React auth state:

1. After login/refresh, the frontend called `GET /me` immediately.
2. The access token was only stored through React state.
3. React state is asynchronous, so the immediate `/me` call could run before the token provider had the new access token.
4. `/me` failed as unauthenticated, so the UI kept `role = null` or stale `role = user`.
5. The frontend route guard blocked `/dashboard` even though the database role had been promoted.

## What changed

### `aegis-console/src/lib/auth/AuthContext.tsx`

- Immediately synchronizes `tokenRef.current` after login.
- Immediately synchronizes `tokenRef.current` after refresh.
- Clears `tokenRef.current` on logout/session clear.

This makes the first `/me` call after login/refresh use the correct bearer token.

### `aegis-console/src/app/RoleRoute.tsx`

- Performs one just-in-time `refreshUser()` before denying an admin route.
- This lets the UI pick up manual DB role changes without forcing a full logout/login.
- Error copy now points to checking `GET /me`, current session validity, and the DB role value.

## How to test

1. Register or login from the UI.
2. Promote the user:

```sql
UPDATE users SET user_role = 'admin' WHERE email = 'your-email@example.com';
```

3. Hard refresh the browser, or navigate to `/dashboard`.
4. `GET /me` should return:

```json
{
  "role": "admin"
}
```

5. The admin dashboard should mount and call:

- `GET /admin/security/events`
- `GET /admin/security/metrics`
- `GET /admin/security/alerts`
- `WS /admin/security/alerts/ws?access_token=...`

## If it still fails

Check the browser network tab:

- `/me` 401: session/token problem. Sign out and login again.
- `/me` role is `user`: DB update did not hit the same database the API is using.
- `/admin/...` 401 with `/me` role `admin`: risk engine is blocking the request, likely due high risk score or IP/session anomaly.
- WebSocket fails only: check that the frontend passes the `access_token` query parameter and that the backend is running the patched WebSocket route.
