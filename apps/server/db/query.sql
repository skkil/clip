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
