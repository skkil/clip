use crate::Line;

pub fn to_text(lines: &[Line]) -> String {
    let mut sorted: Vec<&Line> = lines.iter().collect();
    sorted.sort_by(|a, b| centre_y(a).total_cmp(&centre_y(b)));

    let mut rows: Vec<Vec<&Line>> = Vec::new();
    for line in sorted {
        match rows.last_mut() {
            Some(row) if same_row(row, line) => row.push(line),
            _ => rows.push(vec![line]),
        }
    }

    rows.into_iter()
        .map(|mut row| {
            row.sort_by(|a, b| a.bounds.x.total_cmp(&b.bounds.x));
            row.iter()
                .map(|l| l.text.trim())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn centre_y(line: &Line) -> f32 {
    line.bounds.y + line.bounds.h / 2.0
}

fn same_row(row: &[&Line], line: &Line) -> bool {
    row.iter().any(|other| {
        let top = other.bounds.y.max(line.bounds.y);
        let bottom = (other.bounds.y + other.bounds.h).min(line.bounds.y + line.bounds.h);
        bottom - top > 0.5 * other.bounds.h.min(line.bounds.h)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Box2;

    fn line(text: &str, x: f32, y: f32) -> Line {
        Line {
            text: text.into(),
            score: 1.0,
            bounds: Box2 {
                x,
                y,
                w: 50.0,
                h: 20.0,
            },
            script: "latin".into(),
        }
    }

    #[test]
    fn groups_columns_on_one_row_and_orders_rows() {
        let lines = [
            line("world", 100.0, 2.0),
            line("next", 0.0, 40.0),
            line("hello", 0.0, 0.0),
        ];
        assert_eq!(to_text(&lines), "hello world\nnext");
    }
}
