-- name: CreateRefreshToken :one
INSERT INTO
    refresh_tokens (device_id, user_id, token_hash, expires_at)
VALUES
    ($1, $2, $3, $4)
RETURNING
    id,
    device_id,
    user_id,
    token_hash,
    created_at,
    last_used_at,
    expires_at,
    revoked_at;

-- name: GetRefreshTokenByHash :one
SELECT
    id,
    device_id,
    user_id,
    token_hash,
    created_at,
    last_used_at,
    expires_at,
    revoked_at
FROM
    refresh_tokens
WHERE
    token_hash = $1;

-- name: RevokeRefreshToken :exec
UPDATE refresh_tokens
SET
    revoked_at = NOW()
WHERE
    id = $1;

-- name: CreateAccessToken :one
INSERT INTO
    access_tokens (refresh_token_id, user_id, device_id, token_hash, expires_at)
VALUES
    ($1, $2, $3, $4, $5)
RETURNING
    id,
    refresh_token_id,
    user_id,
    device_id,
    token_hash,
    created_at,
    expires_at;

-- name: GetAccessTokenByHash :one
SELECT
    id,
    refresh_token_id,
    user_id,
    device_id,
    token_hash,
    created_at,
    expires_at
FROM
    access_tokens
WHERE
    token_hash = $1
    AND expires_at > NOW();
