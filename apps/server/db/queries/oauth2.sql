-- name: GetOAuthIdentity :one
SELECT
    id,
    user_id,
    provider,
    provider_user_id,
    created_at
FROM
    oauth_identities
WHERE
    provider = $1
    AND provider_user_id = $2;

-- name: CreateUserWithOAuthIdentity :one
WITH new_user AS (
    INSERT INTO
        users (name, email)
    VALUES
        ($1, $2)
    RETURNING
        id,
        name,
        email,
        created_at
)
INSERT INTO
    oauth_identities (user_id, provider, provider_user_id)
SELECT
    id,
    $3,
    $4
FROM
    new_user
RETURNING
    (
        SELECT
            id
        FROM
            new_user
    ) AS user_id,
    (
        SELECT
            name
        FROM
            new_user
    ) AS name,
    (
        SELECT
            email
        FROM
            new_user
    ) AS email,
    (
        SELECT
            created_at
        FROM
            new_user
    ) AS created_at;
