use clip_ocr::Line;

use crate::geom::{Point, Rect, Side};

pub const ADJUST_STEP: i32 = 5;
pub const CLICK_SLOP: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Format {
    #[default]
    Image,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditTool {
    Redact,
    Mosaic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edit {
    pub tool: EditTool,
    pub area: Rect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shot {
    pub region: Rect,
    pub edits: Vec<Edit>,
    pub cursor: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    Image(Shot),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    Done(Selection),
    Read(Rect),
    Stay,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Editor {
    pub region: Rect,
    pub edits: Vec<Edit>,
    pub tool: EditTool,
    drawing: Option<Point>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextPicker {
    pub region: Rect,
    pub lines: Vec<Line>,
    pub picked: Vec<bool>,
    anchor: Option<Point>,
}

impl TextPicker {
    fn text(&self) -> String {
        let any = self.picked.contains(&true);
        let chosen: Vec<Line> = self
            .lines
            .iter()
            .zip(&self.picked)
            .filter(|(_, picked)| **picked || !any)
            .map(|(line, _)| line.clone())
            .collect();
        clip_ocr::to_text(&chosen)
    }
}

pub fn line_rect(line: &Line) -> Rect {
    let b = line.bounds;
    Rect::from_corners(
        (b.x as f64, b.y as f64),
        ((b.x + b.w) as f64, (b.y + b.h) as f64),
    )
}

#[derive(Debug, Clone, PartialEq)]
pub enum Phase {
    Idle,
    Dragging { anchor: Point },
    Editing(Editor),
    Reading(Rect),
    Picking(TextPicker),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Drag {
    Region(Rect),
    Click(Point),
}

#[derive(Debug, Clone)]
pub struct SelectionState {
    desktop: Rect,
    phase: Phase,
    ctrl_held: bool,
    shift_held: bool,
    cursor: bool,
    format: Format,
    edit_armed: bool,
}

impl SelectionState {
    pub fn new(desktop: Rect, cursor: bool, format: Format) -> Self {
        Self {
            desktop,
            phase: Phase::Idle,
            ctrl_held: false,
            shift_held: false,
            cursor,
            format,
            edit_armed: false,
        }
    }

    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    pub fn is_choosing(&self) -> bool {
        matches!(self.phase, Phase::Idle | Phase::Dragging { .. })
    }

    pub fn is_dragging(&self) -> bool {
        matches!(self.phase, Phase::Dragging { .. })
    }

    pub fn cursor_shown(&self) -> bool {
        self.cursor
    }

    pub fn toggle_cursor(&mut self) -> bool {
        self.cursor = !self.cursor;
        self.cursor
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub fn set_format(&mut self, format: Format) {
        self.format = format;
    }

    pub fn toggle_format(&mut self) {
        self.format = match self.format {
            Format::Image => Format::Text,
            Format::Text => Format::Image,
        };
    }

    pub fn edit_armed(&self) -> bool {
        self.edit_armed
    }

    pub fn toggle_edit(&mut self) {
        self.edit_armed = !self.edit_armed;
    }

    pub fn set_ctrl(&mut self, held: bool) {
        self.ctrl_held = held;
    }

    pub fn set_shift(&mut self, held: bool) {
        self.shift_held = held;
    }

    fn editor(&self) -> Option<&Editor> {
        match &self.phase {
            Phase::Editing(e) => Some(e),
            _ => None,
        }
    }

    fn editor_mut(&mut self) -> Option<&mut Editor> {
        match &mut self.phase {
            Phase::Editing(e) => Some(e),
            _ => None,
        }
    }

    fn picker_mut(&mut self) -> Option<&mut TextPicker> {
        match &mut self.phase {
            Phase::Picking(p) => Some(p),
            _ => None,
        }
    }

    pub fn region(&self) -> Option<Rect> {
        match &self.phase {
            Phase::Editing(e) => Some(e.region),
            Phase::Reading(region) => Some(*region),
            Phase::Picking(p) => Some(p.region),
            _ => None,
        }
    }

    pub fn edits(&self) -> &[Edit] {
        self.editor().map_or(&[], |e| &e.edits)
    }

    pub fn tool(&self) -> Option<EditTool> {
        self.editor().map(|e| e.tool)
    }

    fn shot(&self, region: Rect) -> Shot {
        Shot {
            region,
            edits: Vec::new(),
            cursor: self.cursor,
        }
    }

    pub fn begin(&mut self, anchor: Point) {
        self.phase = Phase::Dragging { anchor };
    }

    pub fn drag_rect(&self, cursor: Point) -> Option<Rect> {
        let Phase::Dragging { anchor } = self.phase else {
            return None;
        };
        Some(Rect::from_corners(anchor, cursor)).filter(|r| !r.is_empty())
    }

    #[must_use]
    pub fn release(&mut self, cursor: Point) -> Option<Drag> {
        let Phase::Dragging { anchor } = self.phase else {
            return None;
        };
        self.phase = Phase::Idle;
        let r = Rect::from_corners(anchor, cursor);
        Some(if r.w < CLICK_SLOP && r.h < CLICK_SLOP {
            Drag::Click(cursor)
        } else {
            Drag::Region(r)
        })
    }

    #[must_use]
    pub fn complete(&mut self, region: Rect) -> Step {
        let region = region.intersect(&self.desktop);
        if region.is_empty() {
            self.phase = Phase::Idle;
            return Step::Stay;
        }
        match self.format {
            Format::Text => {
                self.phase = Phase::Reading(region);
                Step::Read(region)
            }
            Format::Image if self.edit_armed || self.ctrl_held => {
                self.phase = Phase::Editing(Editor {
                    region,
                    edits: Vec::new(),
                    tool: EditTool::Redact,
                    drawing: None,
                });
                Step::Stay
            }
            Format::Image => {
                self.phase = Phase::Idle;
                Step::Done(Selection::Image(self.shot(region)))
            }
        }
    }

    pub fn show_text(&mut self, lines: Vec<Line>) -> bool {
        let Phase::Reading(region) = self.phase else {
            return false;
        };
        if lines.is_empty() {
            self.phase = Phase::Idle;
            return false;
        }
        self.phase = Phase::Picking(TextPicker {
            region,
            picked: vec![false; lines.len()],
            lines,
            anchor: None,
        });
        true
    }

    #[must_use]
    pub fn confirm(&mut self) -> Option<Selection> {
        match std::mem::replace(&mut self.phase, Phase::Idle) {
            Phase::Editing(editor) => Some(Selection::Image(Shot {
                edits: editor.edits,
                ..self.shot(editor.region)
            })),
            Phase::Picking(picker) => Some(Selection::Text(picker.text())),
            phase => {
                self.phase = phase;
                None
            }
        }
    }

    pub fn select_tool(&mut self, tool: EditTool) {
        if let Some(e) = self.editor_mut() {
            e.tool = tool;
            e.drawing = None;
        }
    }

    pub fn draw_begin(&mut self, at: Point) {
        if let Some(e) = self.editor_mut() {
            e.drawing = Some(at);
        }
    }

    pub fn draw_rect(&self, cursor: Point) -> Option<Edit> {
        let e = self.editor()?;
        let area = Rect::from_corners(e.drawing?, cursor).intersect(&e.region);
        (!area.is_empty()).then_some(Edit { tool: e.tool, area })
    }

    pub fn draw_commit(&mut self, cursor: Point) {
        let edit = self
            .draw_rect(cursor)
            .filter(|e| e.area.w >= CLICK_SLOP && e.area.h >= CLICK_SLOP);
        if let Some(e) = self.editor_mut() {
            e.drawing = None;
            e.edits.extend(edit);
        }
    }

    pub fn undo(&mut self) -> bool {
        self.editor_mut().is_some_and(|e| e.edits.pop().is_some())
    }

    pub fn nudge(&mut self, side: Side) {
        let desktop = self.desktop;
        let step = if self.shift_held {
            -ADJUST_STEP
        } else {
            ADJUST_STEP
        };
        let Some(e) = self.editor_mut() else {
            return;
        };
        let moved = e.region.grow_side(side, step).intersect(&desktop);
        if moved.is_empty() {
            return;
        }
        e.region = moved;
        e.edits.retain_mut(|edit| {
            edit.area = edit.area.intersect(&moved);
            !edit.area.is_empty()
        });
    }

    pub fn picker(&self) -> Option<&TextPicker> {
        match &self.phase {
            Phase::Picking(p) => Some(p),
            _ => None,
        }
    }

    pub fn pick_begin(&mut self, at: Point) {
        if let Some(p) = self.picker_mut() {
            p.anchor = Some(at);
        }
    }

    pub fn pick_rect(&self, cursor: Point) -> Option<Rect> {
        let anchor = self.picker()?.anchor?;
        Some(Rect::from_corners(anchor, cursor)).filter(|r| r.w >= CLICK_SLOP || r.h >= CLICK_SLOP)
    }

    pub fn pick_commit(&mut self, cursor: Point) {
        if self.picker().and_then(|p| p.anchor).is_none() {
            return;
        }
        let swept = self.pick_rect(cursor);
        let adding = self.shift_held;
        let Some(p) = self.picker_mut() else {
            return;
        };
        p.anchor = None;
        let rects: Vec<Rect> = p.lines.iter().map(line_rect).collect();
        match swept {
            Some(swept) => {
                for (picked, rect) in p.picked.iter_mut().zip(&rects) {
                    let hit = !rect.intersect(&swept).is_empty();
                    *picked = hit || (adding && *picked);
                }
            }
            None => {
                let (x, y) = (cursor.0 as i32, cursor.1 as i32);
                if let Some(i) = rects.iter().position(|r| r.contains(x, y)) {
                    p.picked[i] = !p.picked[i];
                }
            }
        }
    }

    pub fn toggle_all(&mut self) {
        if let Some(p) = self.picker_mut() {
            let all = !p.picked.contains(&false);
            p.picked.fill(!all);
        }
    }
}

#[cfg(test)]
mod tests {
    use clip_ocr::Box2;

    use super::*;

    const DESKTOP: Rect = Rect::new(2560, 480, 1920, 1200);
    const SEL: Rect = Rect::new(2700, 600, 100, 100);

    fn state() -> SelectionState {
        SelectionState::new(DESKTOP, false, Format::Image)
    }

    fn editing() -> SelectionState {
        let mut s = state();
        s.toggle_edit();
        assert_eq!(s.complete(SEL), Step::Stay);
        s
    }

    fn draw(s: &mut SelectionState, from: Point, to: Point) {
        s.draw_begin(from);
        s.draw_commit(to);
    }

    fn line(text: &str, x: f32, y: f32) -> Line {
        Line {
            text: text.into(),
            score: 1.0,
            bounds: Box2 {
                x,
                y,
                w: 80.0,
                h: 20.0,
            },
            script: "latin".into(),
        }
    }

    fn picking() -> SelectionState {
        let mut s = SelectionState::new(DESKTOP, false, Format::Text);
        assert_eq!(s.complete(SEL), Step::Read(SEL));
        assert!(s.show_text(vec![
            line("first", 2700.0, 600.0),
            line("second", 2700.0, 630.0),
            line("third", 2700.0, 660.0),
        ]));
        s
    }

    fn click(s: &mut SelectionState, at: Point) {
        s.pick_begin(at);
        s.pick_commit(at);
    }

    fn copied(s: &mut SelectionState) -> String {
        match s.confirm() {
            Some(Selection::Text(text)) => text,
            other => panic!("expected text, got {other:?}"),
        }
    }

    #[test]
    fn a_selection_copies_immediately_by_default() {
        let mut s = state();
        let Step::Done(Selection::Image(shot)) = s.complete(SEL) else {
            panic!("expected an image");
        };
        assert_eq!(shot.region, SEL);
        assert_eq!(*s.phase(), Phase::Idle);
    }

    #[test]
    fn edit_armed_opens_the_editor_with_black_picked() {
        let s = editing();
        assert_eq!(s.region(), Some(SEL));
        assert_eq!(s.tool(), Some(EditTool::Redact));
    }

    #[test]
    fn ctrl_held_through_the_gesture_also_opens_the_editor() {
        let mut s = state();
        s.set_ctrl(true);
        assert_eq!(s.complete(SEL), Step::Stay);
        s.set_ctrl(false);
        assert_eq!(s.region(), Some(SEL));
    }

    #[test]
    fn a_selection_outside_the_desktop_is_rejected() {
        let mut s = state();
        assert_eq!(s.complete(Rect::new(0, 0, 10, 10)), Step::Stay);
        assert_eq!(*s.phase(), Phase::Idle);
    }

    #[test]
    fn arrows_grow_and_shift_arrows_shrink_the_same_edge() {
        let mut s = editing();
        s.nudge(Side::Left);
        assert_eq!(s.region(), Some(Rect::new(2695, 600, 105, 100)));
        s.set_shift(true);
        s.nudge(Side::Left);
        assert_eq!(s.region(), Some(SEL));
    }

    #[test]
    fn growing_is_clamped_to_the_desktop() {
        let mut s = state();
        s.toggle_edit();
        let _ = s.complete(Rect::new(2560, 480, 100, 100));
        s.nudge(Side::Left);
        s.nudge(Side::Top);
        assert_eq!(s.region(), Some(Rect::new(2560, 480, 100, 100)));
    }

    #[test]
    fn shrinking_stops_before_the_region_collapses() {
        let mut s = editing();
        s.set_shift(true);
        for _ in 0..100 {
            s.nudge(Side::Right);
        }
        assert!(!s.region().unwrap().is_empty());
    }

    #[test]
    fn the_first_drag_in_the_editor_blacks_out() {
        let mut s = editing();
        draw(&mut s, (2710.0, 610.0), (2760.0, 660.0));
        assert_eq!(s.edits()[0].tool, EditTool::Redact);
    }

    #[test]
    fn confirming_the_editor_returns_the_edits_and_resets() {
        let mut s = editing();
        s.select_tool(EditTool::Mosaic);
        draw(&mut s, (2710.0, 610.0), (2760.0, 660.0));
        let Some(Selection::Image(shot)) = s.confirm() else {
            panic!("expected an image");
        };
        assert_eq!(shot.region, SEL);
        assert_eq!(shot.edits[0].tool, EditTool::Mosaic);
        assert_eq!(*s.phase(), Phase::Idle);
        assert_eq!(s.confirm(), None);
    }

    #[test]
    fn an_edit_is_clamped_to_the_selection() {
        let mut s = editing();
        draw(&mut s, (2750.0, 650.0), (9000.0, 9000.0));
        assert_eq!(s.edits()[0].area, Rect::new(2750, 650, 50, 50));
    }

    #[test]
    fn a_click_without_a_drag_leaves_no_speck() {
        let mut s = editing();
        draw(&mut s, (2710.0, 610.0), (2711.0, 611.0));
        assert!(s.edits().is_empty());
    }

    #[test]
    fn undo_drops_the_most_recent_edit() {
        let mut s = editing();
        draw(&mut s, (2710.0, 610.0), (2740.0, 640.0));
        assert!(s.undo());
        assert!(!s.undo());
    }

    #[test]
    fn shrinking_the_region_trims_the_edits_with_it() {
        let mut s = editing();
        draw(&mut s, (2750.0, 650.0), (2800.0, 700.0));
        s.set_shift(true);
        s.nudge(Side::Right);
        assert!(s.edits()[0].area.right() <= s.region().unwrap().right());
    }

    #[test]
    fn a_press_that_barely_moved_releases_as_a_click() {
        let mut s = state();
        s.begin((2700.0, 600.0));
        assert_eq!(
            s.release((2701.0, 601.0)),
            Some(Drag::Click((2701.0, 601.0)))
        );
        s.begin((2700.0, 600.0));
        assert_eq!(s.release((2800.0, 700.0)), Some(Drag::Region(SEL)));
    }

    #[test]
    fn the_cursor_choice_carries_into_the_shot() {
        let mut s = editing();
        s.toggle_cursor();
        let Some(Selection::Image(shot)) = s.confirm() else {
            panic!("expected an image");
        };
        assert!(shot.cursor);
    }

    #[test]
    fn text_mode_ignores_the_edit_toggle() {
        let mut s = SelectionState::new(DESKTOP, false, Format::Text);
        s.toggle_edit();
        assert_eq!(s.complete(SEL), Step::Read(SEL));
    }

    #[test]
    fn no_text_found_returns_to_choosing() {
        let mut s = SelectionState::new(DESKTOP, false, Format::Text);
        let _ = s.complete(SEL);
        assert!(!s.show_text(Vec::new()));
        assert!(s.is_choosing());
    }

    #[test]
    fn with_nothing_picked_every_line_is_copied() {
        let mut s = picking();
        assert_eq!(copied(&mut s), "first\nsecond\nthird");
    }

    #[test]
    fn clicking_a_line_toggles_it() {
        let mut s = picking();
        click(&mut s, (2710.0, 640.0));
        click(&mut s, (2710.0, 670.0));
        click(&mut s, (2710.0, 670.0));
        assert_eq!(copied(&mut s), "second");
    }

    #[test]
    fn dragging_picks_the_lines_it_touches() {
        let mut s = picking();
        s.pick_begin((2690.0, 605.0));
        s.pick_commit((2720.0, 635.0));
        assert_eq!(copied(&mut s), "first\nsecond");
    }

    #[test]
    fn a_new_drag_replaces_the_pick_unless_shift_is_held() {
        let mut s = picking();
        s.pick_begin((2690.0, 605.0));
        s.pick_commit((2720.0, 615.0));
        s.pick_begin((2690.0, 665.0));
        s.pick_commit((2720.0, 675.0));
        assert_eq!(s.picker().unwrap().picked, [false, false, true]);
        s.set_shift(true);
        s.pick_begin((2690.0, 605.0));
        s.pick_commit((2720.0, 615.0));
        assert_eq!(copied(&mut s), "first\nthird");
    }

    #[test]
    fn toggle_all_picks_everything_then_nothing() {
        let mut s = picking();
        s.toggle_all();
        assert_eq!(s.picker().unwrap().picked, [true; 3]);
        s.toggle_all();
        assert_eq!(s.picker().unwrap().picked, [false; 3]);
    }
}
