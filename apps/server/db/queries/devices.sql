-- name: CreateDevice :one
INSERT INTO
    devices (device_code_hash, user_code, expires_at)
VALUES
    ($1, $2, $3)
RETURNING
    id,
    user_id,
    device_code_hash,
    user_code,
    status,
    name,
    expires_at,
    approved_at,
    created_at;

-- name: GetDeviceByDeviceCodeHash :one
SELECT
    id,
    user_id,
    device_code_hash,
    user_code,
    status,
    name,
    expires_at,
    approved_at,
    created_at
FROM
    devices
WHERE
    device_code_hash = $1;

-- name: GetDeviceByUserCode :one
SELECT
    id,
    user_id,
    device_code_hash,
    user_code,
    status,
    name,
    expires_at,
    approved_at,
    created_at
FROM
    devices
WHERE
    user_code = $1;

-- name: ApproveDevice :one
UPDATE devices
SET
    user_id = $2,
    status = 'approved',
    approved_at = NOW()
WHERE
    id = $1
RETURNING
    id,
    user_id,
    device_code_hash,
    user_code,
    status,
    name,
    expires_at,
    approved_at,
    created_at;
