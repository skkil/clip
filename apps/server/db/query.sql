-- name: CreateUser :one
INSERT INTO
    users (name, email)
VALUES
    ($1, $2)
RETURNING
    id,
    name,
    email,
    created_at;

-- name: CreateClip :one
INSERT INTO
    clips (content)
VALUES
    ($1)
RETURNING
    id,
    content,
    created_at;

-- name: ListClips :many
SELECT
    id,
    content,
    created_at
FROM
    clips
ORDER BY
    created_at DESC
LIMIT
    $1;
