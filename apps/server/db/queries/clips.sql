-- name: CreateClip :one
INSERT INTO
    clips (user_id, content)
VALUES
    ($1, $2)
RETURNING
    id,
    user_id,
    content,
    created_at;

-- name: ListClips :many
SELECT
    id,
    user_id,
    content,
    created_at
FROM
    clips
WHERE
    user_id = $1
ORDER BY
    created_at DESC
LIMIT
    $2;
