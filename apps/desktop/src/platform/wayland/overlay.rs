use std::sync::Arc;

use anyhow::{Context, Result};
use smithay_client_toolkit::compositor::{CompositorHandler, CompositorState, FrameCallbackData};
use smithay_client_toolkit::dispatch2::Dispatch2;
use smithay_client_toolkit::output::{OutputHandler, OutputState};
use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
use smithay_client_toolkit::seat::keyboard::{
    KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers,
};
use smithay_client_toolkit::seat::pointer::{
    CursorIcon, PointerEvent, PointerEventKind, PointerHandler, ThemeSpec, ThemedPointer,
};
use smithay_client_toolkit::seat::{Capability, SeatHandler, SeatState};
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shell::wlr_layer::{
    Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
    LayerSurfaceConfigure,
};
use smithay_client_toolkit::shm::slot::SlotPool;
use smithay_client_toolkit::shm::{Shm, ShmHandler};
use smithay_client_toolkit::{delegate_registry, registry_handlers};
use wayland_client::globals::registry_queue_init;
use wayland_client::protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm, wl_surface};
use wayland_client::{Connection, Proxy, QueueHandle};
use wayland_protocols::wp::viewporter::client::{
    wp_viewport::WpViewport, wp_viewporter::WpViewporter,
};

use super::windows;
use crate::capture::{Capture, CursorLayer, MOSAIC_BLOCK};
use crate::geom::{Point, Rect, Side};
use crate::menu::{MenuMetrics, place_menu};
use crate::platform::{ReadText, SelectMode, SelectOptions, WindowInfo, window_at};
use crate::raster::{Image, Rgba};
use crate::select::{
    Drag, Edit, EditTool, Format, Phase, Selection, SelectionState, Step, TextPicker, line_rect,
};
use crate::text::{TextRenderer, text_top};

const BTN_RIGHT: u32 = 0x111;
const DIM: f32 = 0.55;
const INK: Rgba = [20, 24, 26, 238];
const INK_TEXT: Rgba = [20, 24, 26, 255];
const PAPER: Rgba = [233, 236, 234, 255];
const MUTED: Rgba = [154, 166, 163, 255];
const DISABLED: Rgba = [96, 106, 104, 255];
const ROUTE: Rgba = [255, 90, 79, 255];
const PICKED: Rgba = [255, 90, 79, 84];
const UNPICKED: Rgba = [233, 236, 234, 40];
const HOVER: Rgba = [233, 236, 234, 26];
const SEPARATOR: Rgba = [233, 236, 234, 46];
const GUIDE: Rgba = [233, 236, 234, 70];
const REDACT: Rgba = [0, 0, 0, 255];
const MAG_SIZE: i32 = 140;
const MAG_ZOOM: i32 = 8;
const BAR_HEIGHT: i32 = 40;
const BAR_INSET: i32 = 4;
const BAR_GAP: i32 = 12;
const BUTTON_PAD_X: i32 = 14;
const GROUP_GAP: i32 = 9;
const KEY_GAP: i32 = 6;
const LABEL_SIZE: u32 = 14;
const KEY_SIZE: u32 = 11;
const HINT_SIZE: u32 = 13;
const BADGE_SIZE: u32 = 13;
const TOOLBAR_TOP: i32 = 20;
const BRACKET: i32 = 22;
const NO_TEXT: &str = "No text found here. Try a larger area, or Esc to cancel";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Button {
    Mode(SelectMode),
    Format(Format),
    Cursor,
    Edit,
    Tool(EditTool),
    Undo,
    All,
    Copy,
}

impl Button {
    fn label(self) -> &'static str {
        match self {
            Button::Mode(SelectMode::Region) => "Region",
            Button::Mode(SelectMode::Screen) => "Screen",
            Button::Format(Format::Image) => "Image",
            Button::Format(Format::Text) => "Text",
            Button::Cursor => "Cursor",
            Button::Edit => "Edit",
            Button::Tool(EditTool::Redact) => "Black",
            Button::Tool(EditTool::Mosaic) => "Mosaic",
            Button::Undo => "Undo",
            Button::All => "All",
            Button::Copy => "Copy",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Button::Mode(_) => "Tab",
            Button::Format(_) => "T",
            Button::Cursor => "C",
            Button::Edit => "E",
            Button::Tool(EditTool::Redact) => "1",
            Button::Tool(EditTool::Mosaic) => "2",
            Button::Undo => "U",
            Button::All => "A",
            Button::Copy => "Enter",
        }
    }
}

type Groups = &'static [&'static [Button]];

const TOOLBAR: Groups = &[
    &[
        Button::Mode(SelectMode::Region),
        Button::Mode(SelectMode::Screen),
    ],
    &[Button::Format(Format::Image), Button::Format(Format::Text)],
    &[Button::Cursor],
    &[Button::Edit],
];
const EDITOR_MENU: Groups = &[
    &[
        Button::Tool(EditTool::Redact),
        Button::Tool(EditTool::Mosaic),
    ],
    &[Button::Undo],
    &[Button::Copy],
];
const PICKER_MENU: Groups = &[&[Button::All], &[Button::Copy]];

struct Bar {
    rect: Rect,
    buttons: Vec<(Button, Rect)>,
    separators: Vec<i32>,
}

struct Surf {
    name: String,
    layer: LayerSurface,
    viewport: Option<WpViewport>,
    logical: Rect,
    scale: f64,
    bright: Arc<Image>,
    dimmed: Image,
    plain: Arc<Image>,
    cursor: Option<CursorLayer>,
    pool: SlotPool,
    format: wl_shm::Format,
    canvas: Option<Image>,
    configured: bool,
    needs_draw: bool,
    frame_pending: bool,
}

impl Surf {
    fn to_local(&self, (x, y): Point) -> (i32, i32) {
        (
            ((x - self.logical.x as f64) * self.scale).round() as i32,
            ((y - self.logical.y as f64) * self.scale).round() as i32,
        )
    }

    fn rect_to_local(&self, r: Rect) -> Rect {
        r.relative_to((self.logical.x, self.logical.y))
            .scaled(self.scale)
    }

    fn rect_to_global(&self, r: Rect) -> Rect {
        let s = r.scaled(1.0 / self.scale);
        Rect::new(s.x + self.logical.x, s.y + self.logical.y, s.w, s.h)
    }

    fn px(&self, v: i32) -> i32 {
        ((v as f64 * self.scale).round() as i32).max(1)
    }

    fn upx(&self, v: u32) -> u32 {
        self.px(v as i32) as u32
    }

    fn size(&self) -> (u32, u32) {
        (self.bright.width, self.bright.height)
    }

    fn show_cursor(&mut self, shown: bool) {
        let Some(layer) = &self.cursor else {
            return;
        };
        let patch = if shown {
            Ok(layer.image.clone())
        } else {
            self.plain.crop(layer.area)
        };
        let Ok(mut patch) = patch else {
            return;
        };
        let (x, y) = (layer.area.x, layer.area.y);
        Arc::make_mut(&mut self.bright).blend(&patch, x, y);
        patch.darken(DIM);
        self.dimmed.blend(&patch, x, y);
        self.needs_draw = true;
    }
}

struct App {
    registry: RegistryState,
    outputs: OutputState,
    seats: SeatState,
    shm: Shm,
    compositor: CompositorState,
    surfaces: Vec<Surf>,
    pointer: Option<ThemedPointer>,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    cursor_icon: CursorIcon,
    cursor: Option<Point>,
    sel: SelectionState,
    windows: Vec<WindowInfo>,
    mode: SelectMode,
    zoom_held: bool,
    notice: Option<&'static str>,
    text: TextRenderer,
    exit: bool,
    outcome: Option<Selection>,
}

impl App {
    fn mark_dirty(&mut self) {
        for s in &mut self.surfaces {
            s.needs_draw = true;
        }
    }

    fn surface(&self, wl: &wl_surface::WlSurface) -> Option<&Surf> {
        self.surfaces.iter().find(|s| s.layer.wl_surface() == wl)
    }

    fn surface_mut(&mut self, wl: &wl_surface::WlSurface) -> Option<&mut Surf> {
        self.surfaces
            .iter_mut()
            .find(|s| s.layer.wl_surface() == wl)
    }

    fn live_region(&self) -> Option<Rect> {
        match self.sel.phase() {
            Phase::Dragging { .. } if self.mode == SelectMode::Region => {
                self.cursor.and_then(|c| self.sel.drag_rect(c))
            }
            Phase::Idle | Phase::Dragging { .. } => self.hovered_target(),
            _ => self.sel.region(),
        }
    }

    fn target_at(&self, (x, y): Point) -> Option<Rect> {
        match self.mode {
            SelectMode::Region => window_at(&self.windows, x as i32, y as i32).map(|w| w.rect),
            SelectMode::Screen => self.screen_at(x, y).map(|s| s.logical),
        }
    }

    fn hovered_target(&self) -> Option<Rect> {
        let pos = self.cursor?;
        if self.button_at(pos).is_some() {
            return None;
        }
        self.target_at(pos)
    }

    fn screen_at(&self, x: f64, y: f64) -> Option<&Surf> {
        self.surfaces
            .iter()
            .find(|s| s.logical.contains(x as i32, y as i32))
    }

    fn host(&self) -> Option<usize> {
        let (cx, cy) = self.sel.region()?.centre();
        self.surfaces
            .iter()
            .position(|s| s.logical.contains(cx, cy))
    }

    fn groups(&self) -> Option<Groups> {
        match self.sel.phase() {
            Phase::Idle => Some(TOOLBAR),
            Phase::Editing(_) => Some(EDITOR_MENU),
            Phase::Picking(_) => Some(PICKER_MENU),
            _ => None,
        }
    }

    fn shows_key(&self, button: Button) -> bool {
        !matches!(button, Button::Mode(_) | Button::Format(_)) || !self.button_on(button)
    }

    fn button_width(&self, surf: &Surf, button: Button) -> i32 {
        let (label, _) = self.text.measure(button.label(), surf.upx(LABEL_SIZE));
        let mut width = label as i32 + surf.px(BUTTON_PAD_X) * 2;
        if self.shows_key(button) {
            let (key, _) = self.text.measure(button.key(), surf.upx(KEY_SIZE));
            width += key as i32 + surf.px(KEY_GAP);
        }
        width
    }

    fn bar(&self, index: usize) -> Option<Bar> {
        let surf = &self.surfaces[index];
        let groups = self.groups()?;
        let inset = surf.px(BAR_INSET);
        let gap = surf.px(GROUP_GAP);
        let height = surf.px(BAR_HEIGHT);

        let mut x = inset;
        let mut buttons = Vec::new();
        let mut separators = Vec::new();
        for (i, group) in groups.iter().enumerate() {
            if i > 0 {
                separators.push(x + gap / 2);
                x += gap;
            }
            for &button in *group {
                let w = self.button_width(surf, button);
                buttons.push((
                    button,
                    Rect::new(x, inset, w as u32, (height - inset * 2) as u32),
                ));
                x += w;
            }
        }
        let size = Rect::new(0, 0, (x + inset) as u32, height as u32);

        let rect = match self.sel.region() {
            Some(region) if self.host() == Some(index) => {
                let metrics = MenuMetrics {
                    height,
                    gap: surf.px(BAR_GAP),
                    min_width: size.w,
                    max_width: size.w,
                };
                place_menu(surf.rect_to_local(region), surf.size(), metrics)
            }
            Some(_) => return None,
            None => Rect::new(
                (surf.size().0 as i32 - size.w as i32) / 2,
                surf.px(TOOLBAR_TOP),
                size.w,
                size.h,
            ),
        };
        let shift = |r: Rect| Rect::new(r.x + rect.x, r.y + rect.y, r.w, r.h);
        Some(Bar {
            rect,
            buttons: buttons.into_iter().map(|(b, r)| (b, shift(r))).collect(),
            separators: separators.into_iter().map(|x| x + rect.x).collect(),
        })
    }

    fn button_at(&self, (x, y): Point) -> Option<Button> {
        let index = self
            .surfaces
            .iter()
            .position(|s| s.logical.contains(x as i32, y as i32))?;
        let surf = &self.surfaces[index];
        self.bar(index)?
            .buttons
            .into_iter()
            .find(|(_, slot)| surf.rect_to_global(*slot).contains(x as i32, y as i32))
            .map(|(button, _)| button)
    }

    fn button_on(&self, button: Button) -> bool {
        match button {
            Button::Mode(mode) => self.mode == mode,
            Button::Format(format) => self.sel.format() == format,
            Button::Cursor => self.sel.cursor_shown(),
            Button::Edit => self.sel.edit_armed() && self.sel.format() == Format::Image,
            Button::Tool(tool) => self.sel.tool() == Some(tool),
            Button::Copy => true,
            Button::Undo | Button::All => false,
        }
    }

    fn button_enabled(&self, button: Button) -> bool {
        match button {
            Button::Edit => self.sel.format() == Format::Image,
            Button::Undo => !self.sel.edits().is_empty(),
            _ => true,
        }
    }

    fn activate(&mut self, button: Button) {
        if !self.button_enabled(button) {
            return;
        }
        match button {
            Button::Mode(mode) => self.mode = mode,
            Button::Format(format) => self.sel.set_format(format),
            Button::Cursor => self.toggle_cursor(),
            Button::Edit => self.sel.toggle_edit(),
            Button::Tool(tool) => self.sel.select_tool(tool),
            Button::Undo => {
                self.sel.undo();
            }
            Button::All => self.sel.toggle_all(),
            Button::Copy => self.confirm(),
        }
    }

    fn toggle_cursor(&mut self) {
        let shown = self.sel.toggle_cursor();
        for s in &mut self.surfaces {
            s.show_cursor(shown);
        }
    }

    fn finish(&mut self, selection: Selection) {
        self.outcome = Some(selection);
        self.exit = true;
    }

    fn complete(&mut self, region: Rect) {
        if let Step::Done(selection) = self.sel.complete(region) {
            self.finish(selection);
        }
    }

    fn confirm(&mut self) {
        if let Some(selection) = self.sel.confirm() {
            self.finish(selection);
        } else if self.sel.is_choosing()
            && let Some(target) = self.hovered_target()
        {
            self.complete(target);
        }
    }

    fn sync_cursor_icon(&mut self, conn: &Connection) {
        let over_button = self.cursor.is_some_and(|c| self.button_at(c).is_some());
        let want = match self.sel.phase() {
            _ if over_button => CursorIcon::Default,
            Phase::Idle | Phase::Dragging { .. } => CursorIcon::Crosshair,
            Phase::Reading(_) => CursorIcon::Wait,
            Phase::Picking(_) => CursorIcon::Text,
            Phase::Editing(_) => CursorIcon::Default,
        };
        if want != self.cursor_icon {
            self.cursor_icon = want;
            if let Some(p) = &self.pointer {
                p.set_cursor(conn, want).ok();
            }
        }
    }

    fn press(&mut self, pos: Point) {
        self.cursor = Some(pos);
        self.notice = None;
        if let Some(button) = self.button_at(pos) {
            self.activate(button);
        } else {
            match self.sel.phase() {
                Phase::Editing(_) => self.sel.draw_begin(pos),
                Phase::Picking(_) => self.sel.pick_begin(pos),
                Phase::Reading(_) => {}
                Phase::Idle | Phase::Dragging { .. } => self.sel.begin(pos),
            }
        }
        self.mark_dirty();
    }

    fn release(&mut self, pos: Point) {
        match self.sel.phase() {
            Phase::Editing(_) => self.sel.draw_commit(pos),
            Phase::Picking(_) => self.sel.pick_commit(pos),
            _ => match self.sel.release(pos) {
                Some(Drag::Region(region)) if self.mode == SelectMode::Region => {
                    self.complete(region)
                }
                Some(_) => {
                    if let Some(target) = self.target_at(pos) {
                        self.complete(target);
                    }
                }
                None => {}
            },
        }
        self.mark_dirty();
    }

    fn key(&mut self, keysym: Keysym) {
        self.notice = None;
        match keysym {
            Keysym::Escape | Keysym::q => {
                self.outcome = None;
                self.exit = true;
            }
            Keysym::Tab => self.mode = self.mode.next(),
            Keysym::t | Keysym::T => self.sel.toggle_format(),
            Keysym::c | Keysym::C => self.toggle_cursor(),
            Keysym::e | Keysym::E => self.activate(Button::Edit),
            Keysym::a | Keysym::A => self.sel.toggle_all(),
            Keysym::_1 => self.sel.select_tool(EditTool::Redact),
            Keysym::_2 => self.sel.select_tool(EditTool::Mosaic),
            Keysym::BackSpace | Keysym::u => {
                self.sel.undo();
            }
            Keysym::z | Keysym::Z => self.zoom_held = true,
            Keysym::Left => self.sel.nudge(Side::Left),
            Keysym::Right => self.sel.nudge(Side::Right),
            Keysym::Up => self.sel.nudge(Side::Top),
            Keysym::Down => self.sel.nudge(Side::Bottom),
            Keysym::Return | Keysym::KP_Enter | Keysym::space => self.confirm(),
            _ => return,
        }
        self.mark_dirty();
    }

    fn hint(&self) -> String {
        if let Some(notice) = self.notice {
            return notice.to_owned();
        }
        let hint = match (self.sel.phase(), self.mode, self.sel.format()) {
            (Phase::Editing(_), ..) => {
                "Drag to cover  ·  Arrows resize, Shift+arrows shrink  ·  Enter to copy  ·  Esc to cancel"
            }
            (Phase::Reading(_), ..) => "Reading text…",
            (Phase::Picking(picker), ..) => {
                let n = picker.picked.iter().filter(|p| **p).count();
                let copies = match n {
                    0 => "all of it".to_owned(),
                    1 => "1 line".to_owned(),
                    n => format!("{n} lines"),
                };
                return format!(
                    "Click or drag over lines to pick them  ·  Shift adds  ·  Enter copies {copies}  ·  Esc to cancel"
                );
            }
            (_, SelectMode::Screen, _) => "Click a display  ·  Esc to cancel",
            (_, SelectMode::Region, Format::Text) => {
                "Drag over the text you want  ·  Click a window  ·  Esc to cancel"
            }
            (_, SelectMode::Region, Format::Image) if self.sel.edit_armed() => {
                "Drag or click a window, then edit  ·  Hold Z to zoom  ·  Esc to cancel"
            }
            (_, SelectMode::Region, Format::Image) => {
                "Drag to capture an area  ·  Click a window  ·  Hold Z to zoom  ·  Esc to cancel"
            }
        };
        hint.to_owned()
    }

    fn hover_label(&self, surf: &Surf) -> Option<(Rect, String)> {
        let pos = self.cursor?;
        if self.button_at(pos).is_some() {
            return None;
        }
        match self.mode {
            SelectMode::Region => {
                let w = window_at(&self.windows, pos.0 as i32, pos.1 as i32)
                    .filter(|w| !w.title.is_empty())?;
                let label = if w.app_id.is_empty() {
                    w.title.clone()
                } else {
                    format!("{} - {}", w.app_id, w.title)
                };
                Some((w.rect, label))
            }
            SelectMode::Screen => {
                let r = surf.logical;
                Some((r, format!("{}  {}x{}", surf.name, r.w, r.h)))
            }
        }
    }

    fn render(&mut self, index: usize, canvas: &mut Image) {
        let choosing = self.sel.is_choosing();
        let dragging = self.sel.is_dragging();
        let idle = matches!(self.sel.phase(), Phase::Idle);
        let bar = self.bar(index).map(|bar| {
            let states: Vec<_> = bar
                .buttons
                .iter()
                .map(|&(b, slot)| ButtonState {
                    button: b,
                    slot,
                    on: self.button_on(b),
                    enabled: self.button_enabled(b),
                    key: self.shows_key(b),
                })
                .collect();
            (bar, states)
        });
        let hovered_button = self.cursor.and_then(|c| self.button_at(c));
        let hint = (!dragging).then(|| self.hint());

        let mut text = std::mem::take(&mut self.text);
        let surf = &self.surfaces[index];
        let region = self.live_region();
        let cursor = self
            .cursor
            .filter(|&(x, y)| surf.logical.contains(x as i32, y as i32));
        let local = region.map(|r| surf.rect_to_local(r));

        compose(canvas, &surf.dimmed, &surf.bright, local);

        if let (Some(r), Some(local)) = (region, local) {
            for edit in self.sel.edits() {
                apply_edit(canvas, surf, *edit);
            }
            if let Some(edit) = self.cursor.and_then(|c| self.sel.draw_rect(c)) {
                apply_edit(canvas, surf, edit);
                canvas.stroke_rect(surf.rect_to_local(edit.area), ROUTE, surf.upx(1));
            }
            if let Some(picker) = self.sel.picker() {
                draw_lines(canvas, surf, picker);
            }
            if let Some(swept) = self.cursor.and_then(|c| self.sel.pick_rect(c)) {
                canvas.stroke_rect(surf.rect_to_local(swept), ROUTE, surf.upx(1));
            }
            draw_frame(canvas, surf, local);

            if matches!(self.sel.phase(), Phase::Reading(_)) && self.host() == Some(index) {
                let size = surf.upx(15);
                let label = "Reading text…";
                let (w, h) = text.pill_size(label, size);
                let (cx, cy) = (local.x + local.w as i32 / 2, local.y + local.h as i32 / 2);
                text.draw_pill(
                    canvas,
                    label,
                    (cx - w as i32 / 2, cy - h as i32 / 2),
                    size,
                    PAPER,
                    INK,
                );
            }

            if choosing || matches!(self.sel.phase(), Phase::Editing(_)) {
                let size_label = format!("{} × {}", r.w, r.h);
                let label = match self.hover_label(surf).filter(|_| idle) {
                    Some((_, title)) => {
                        let title: String = title.chars().take(60).collect();
                        format!("{title}   {size_label}")
                    }
                    None => size_label,
                };
                let size = surf.upx(BADGE_SIZE);
                let (_, h) = text.pill_size(&label, size);
                let gap = surf.px(8);
                let y = if local.y > h as i32 + gap {
                    local.y - h as i32 - gap
                } else {
                    local.y + gap
                };
                let x = if local.y > h as i32 + gap {
                    local.x
                } else {
                    local.x + gap
                };
                text.draw_pill(canvas, &label, (x, y), size, PAPER, INK);
            }
        }

        if let Some(pos) = cursor
            && choosing
            && !dragging
            && self.mode == SelectMode::Region
            && hovered_button.is_none()
        {
            let (x, y) = surf.to_local(pos);
            canvas.crosshair(x, y, GUIDE);
        }

        if let Some(pos) = cursor
            && self.zoom_held
        {
            draw_magnifier(canvas, surf, surf.to_local(pos), &mut text);
        }

        let mut hint_y = surf.px(TOOLBAR_TOP);
        if let Some((bar, states)) = &bar {
            draw_bar(canvas, surf, bar, states, hovered_button, &mut text);
            if idle {
                hint_y = bar.rect.bottom() + surf.px(8);
            }
        }

        if let Some(hint) = hint {
            let size = surf.upx(HINT_SIZE);
            let (w, _) = text.pill_size(&hint, size);
            let x = (canvas.width as i32 - w as i32) / 2;
            let fg = if self.notice.is_some() { PAPER } else { MUTED };
            text.draw_pill(canvas, &hint, (x, hint_y), size, fg, INK);
        }
        self.text = text;
    }

    fn draw(&mut self, qh: &QueueHandle<Self>, index: usize) {
        let (width, height) = self.surfaces[index].size();
        let mut canvas = match self.surfaces[index].canvas.take() {
            Some(c) if (c.width, c.height) == (width, height) => c,
            _ => Image::new(width, height),
        };
        self.render(index, &mut canvas);

        let surf = &mut self.surfaces[index];
        let stride = width as i32 * 4;
        let Ok((buffer, data)) =
            surf.pool
                .create_buffer(width as i32, height as i32, stride, surf.format)
        else {
            return;
        };
        if surf.format == wl_shm::Format::Abgr8888 {
            data.copy_from_slice(&canvas.pixels);
        } else {
            for (dst, src) in data
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(canvas.pixels.as_chunks::<4>().0)
            {
                *dst = [src[2], src[1], src[0], 255];
            }
        }
        let wl = surf.layer.wl_surface();
        wl.damage_buffer(0, 0, width as i32, height as i32);
        wl.frame(qh, FrameCallbackData(wl.clone()));
        if buffer.attach_to(wl).is_ok() {
            wl.commit();
        }
        surf.canvas = Some(canvas);
        surf.needs_draw = false;
        surf.frame_pending = true;
    }
}

fn compose(canvas: &mut Image, dimmed: &Image, bright: &Image, region: Option<Rect>) {
    let stride = canvas.stride();
    let Some(r) = region
        .map(|r| r.intersect(&canvas.rect()))
        .filter(|r| !r.is_empty())
    else {
        canvas.pixels.copy_from_slice(&dimmed.pixels);
        return;
    };
    let (top, bottom) = (r.y as usize * stride, r.bottom() as usize * stride);
    let (x0, x1) = (r.x as usize * 4, r.right() as usize * 4);
    canvas.pixels[..top].copy_from_slice(&dimmed.pixels[..top]);
    canvas.pixels[bottom..].copy_from_slice(&dimmed.pixels[bottom..]);
    for row in (top..bottom).step_by(stride) {
        let (start, end) = (row + x0, row + x1);
        canvas.pixels[row..start].copy_from_slice(&dimmed.pixels[row..start]);
        canvas.pixels[start..end].copy_from_slice(&bright.pixels[start..end]);
        canvas.pixels[end..row + stride].copy_from_slice(&dimmed.pixels[end..row + stride]);
    }
}

fn apply_edit(canvas: &mut Image, surf: &Surf, edit: Edit) {
    let r = surf.rect_to_local(edit.area);
    match edit.tool {
        EditTool::Redact => canvas.fill_rect(r, REDACT),
        EditTool::Mosaic => canvas.pixelate(r, surf.upx(MOSAIC_BLOCK)),
    }
}

fn draw_frame(canvas: &mut Image, surf: &Surf, r: Rect) {
    canvas.stroke_rect(r, ROUTE, surf.upx(2));
    let t = surf.px(4);
    let len = surf.px(BRACKET).min(r.w as i32 / 3).min(r.h as i32 / 3);
    if len <= t {
        return;
    }
    let (x0, y0, x1, y1) = (r.x, r.y, r.right(), r.bottom());
    let (t, l) = (t as u32, len as u32);
    for (x, y, w, h) in [
        (x0, y0, l, t),
        (x0, y0, t, l),
        (x1 - len, y0, l, t),
        (x1 - t as i32, y0, t, l),
        (x0, y1 - t as i32, l, t),
        (x0, y1 - len, t, l),
        (x1 - len, y1 - t as i32, l, t),
        (x1 - t as i32, y1 - len, t, l),
    ] {
        canvas.fill_rect(Rect::new(x, y, w, h), ROUTE);
    }
}

fn draw_lines(canvas: &mut Image, surf: &Surf, picker: &TextPicker) {
    for (line, picked) in picker.lines.iter().zip(&picker.picked) {
        let r = surf.rect_to_local(line_rect(line));
        canvas.fill_rounded(r, surf.upx(3), if *picked { PICKED } else { UNPICKED });
    }
}

struct ButtonState {
    button: Button,
    slot: Rect,
    on: bool,
    enabled: bool,
    key: bool,
}

fn draw_bar(
    canvas: &mut Image,
    surf: &Surf,
    bar: &Bar,
    states: &[ButtonState],
    hovered: Option<Button>,
    text: &mut TextRenderer,
) {
    canvas.fill_rounded(bar.rect, bar.rect.h / 2, INK);
    let margin = surf.px(12);
    for &x in &bar.separators {
        let h = (bar.rect.h as i32 - margin * 2).max(1) as u32;
        canvas.fill_rect(Rect::new(x, bar.rect.y + margin, surf.upx(1), h), SEPARATOR);
    }
    let (size, key_size) = (surf.upx(LABEL_SIZE), surf.upx(KEY_SIZE));
    for state in states {
        let slot = state.slot;
        let hover = hovered == Some(state.button) && state.enabled;
        let (bg, fg, key_fg) = if state.button == Button::Copy {
            (Some(ROUTE), PAPER, [255, 218, 214, 255])
        } else if state.on {
            (Some(PAPER), INK_TEXT, [84, 94, 92, 255])
        } else if hover {
            (Some(HOVER), PAPER, MUTED)
        } else if state.enabled {
            (None, PAPER, MUTED)
        } else {
            (None, DISABLED, DISABLED)
        };
        if let Some(bg) = bg {
            canvas.fill_rounded(slot, slot.h / 2, bg);
        }
        let label = state.button.label();
        let x = slot.x + surf.px(BUTTON_PAD_X);
        let y = text_top(slot.y + slot.h as i32 / 2, size);
        text.draw(canvas, label, x, y, size, fg);
        if state.key {
            let (lw, _) = text.measure(label, size);
            let kx = x + lw as i32 + surf.px(KEY_GAP);
            let ky = y + size as i32 - key_size as i32;
            text.draw(canvas, state.button.key(), kx, ky, key_size, key_fg);
        }
    }
}

fn draw_magnifier(canvas: &mut Image, surf: &Surf, (px, py): (i32, i32), text: &mut TextRenderer) {
    let size = surf.px(MAG_SIZE);
    let span = (size / MAG_ZOOM).max(1);
    let gap = surf.px(24);
    let src = Rect::new(px - span / 2, py - span / 2, span as u32, span as u32);
    let dx = if px + gap + size > canvas.width as i32 {
        px - gap - size
    } else {
        px + gap
    };
    let dy = if py + gap + size > canvas.height as i32 {
        py - gap - size
    } else {
        py + gap
    };
    let dst = Rect::new(dx, dy, size as u32, size as u32);
    canvas.fill_rect(dst, [0, 0, 0, 255]);
    canvas.blit_magnified(&surf.bright, src, dst);
    canvas.stroke_rect(dst, PAPER, surf.upx(2));
    let centre = Rect::new(
        dx + (size - MAG_ZOOM) / 2,
        dy + (size - MAG_ZOOM) / 2,
        MAG_ZOOM as u32,
        MAG_ZOOM as u32,
    );
    canvas.stroke_rect(centre, [255, 255, 255, 220], 1);

    if surf.bright.rect().contains(px, py) {
        let [r, g, b, _] = surf.bright.pixel(px as u32, py as u32);
        let label = format!("#{r:02X}{g:02X}{b:02X}");
        let at = (dx, dy + size + surf.px(6));
        text.draw_pill(canvas, &label, at, surf.upx(BADGE_SIZE), PAPER, INK);
    }
}

pub fn select(
    capture: &Capture,
    opts: SelectOptions,
    read: &ReadText,
) -> Result<Option<Selection>> {
    let conn = Connection::connect_to_env().context("connect to wayland display")?;
    let (globals, mut queue) = registry_queue_init(&conn).context("registry init")?;
    let qh = queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).context("no wl_compositor")?;
    let layer_shell = LayerShell::bind(&globals, &qh)
        .context("compositor does not support zwlr_layer_shell_v1")?;
    let viewporter: Option<WpViewporter> = globals.bind(&qh, 1..=1, ViewporterData).ok();

    let mut app = App {
        registry: RegistryState::new(&globals),
        outputs: OutputState::new(&globals, &qh),
        seats: SeatState::new(&globals, &qh),
        shm: Shm::bind(&globals, &qh).context("no wl_shm")?,
        compositor: compositor.clone(),
        surfaces: Vec::new(),
        pointer: None,
        keyboard: None,
        cursor_icon: CursorIcon::Crosshair,
        cursor: None,
        sel: SelectionState::new(capture.bounds(), opts.cursor, opts.format),
        windows: windows::list(),
        mode: opts.mode,
        zoom_held: false,
        notice: None,
        text: TextRenderer::new(),
        exit: false,
        outcome: None,
    };
    queue.roundtrip(&mut app).context("list outputs")?;
    queue.roundtrip(&mut app).context("read output geometry")?;

    let format = if app.shm.formats().contains(&wl_shm::Format::Abgr8888) {
        wl_shm::Format::Abgr8888
    } else {
        wl_shm::Format::Argb8888
    };
    let wl_outputs: Vec<_> = app.outputs.outputs().collect();
    for screen in &capture.screens {
        let wl_output = wl_outputs.iter().find(|o| {
            app.outputs.info(o).and_then(|i| i.name).as_deref() == Some(screen.name.as_str())
        });
        let surface = compositor.create_surface(&qh);
        let viewport = viewporter
            .as_ref()
            .map(|v| v.get_viewport(&surface, &qh, ViewporterData));
        let layer = layer_shell.create_layer_surface(
            &qh,
            surface,
            Layer::Overlay,
            Some("clip-select"),
            wl_output,
        );
        layer.set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
        layer.set_exclusive_zone(-1);
        layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
        layer.set_size(0, 0);
        layer.commit();

        let mut dimmed = (*screen.image).clone();
        dimmed.darken(DIM);
        let pool = SlotPool::new(screen.image.pixels.len().max(4096), &app.shm)
            .context("create overlay shm pool")?;
        let mut surf = Surf {
            name: screen.name.clone(),
            layer,
            viewport,
            logical: screen.logical,
            scale: screen.scale,
            bright: Arc::clone(&screen.image),
            dimmed,
            plain: Arc::clone(&screen.image),
            cursor: screen.cursor.clone(),
            pool,
            format,
            canvas: None,
            configured: false,
            needs_draw: true,
            frame_pending: false,
        };
        if opts.cursor {
            surf.show_cursor(true);
        }
        app.surfaces.push(surf);
    }

    let mut failure = None;
    while !app.exit {
        queue
            .blocking_dispatch(&mut app)
            .context("dispatch overlay")?;
        for i in 0..app.surfaces.len() {
            let s = &app.surfaces[i];
            if s.configured && s.needs_draw && !s.frame_pending {
                app.draw(&qh, i);
            }
        }
        if let Phase::Reading(region) = *app.sel.phase()
            && app.surfaces.iter().all(|s| !s.needs_draw)
        {
            conn.flush().ok();
            match read(region) {
                Ok(lines) => {
                    if !app.sel.show_text(lines) {
                        app.notice = Some(NO_TEXT);
                    }
                    app.mark_dirty();
                    app.sync_cursor_icon(&conn);
                }
                Err(e) => {
                    failure = Some(e);
                    break;
                }
            }
        }
    }

    for s in &app.surfaces {
        if let Some(vp) = &s.viewport {
            vp.destroy();
        }
        s.layer.wl_surface().attach(None, 0, 0);
        s.layer.wl_surface().commit();
    }
    app.surfaces.clear();
    conn.roundtrip().ok();
    match failure {
        Some(e) => Err(e),
        None => Ok(app.outcome),
    }
}

struct ViewporterData;

impl Dispatch2<WpViewporter, App> for ViewporterData {
    fn event(
        &self,
        _: &mut App,
        _: &WpViewporter,
        _: <WpViewporter as Proxy>::Event,
        _: &Connection,
        _: &QueueHandle<App>,
    ) {
    }
}

impl Dispatch2<WpViewport, App> for ViewporterData {
    fn event(
        &self,
        _: &mut App,
        _: &WpViewport,
        _: <WpViewport as Proxy>::Event,
        _: &Connection,
        _: &QueueHandle<App>,
    ) {
    }
}

impl CompositorHandler for App {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        if let Some(s) = self.surface_mut(surface) {
            s.frame_pending = false;
        }
    }

    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for App {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        let Some(s) = self.surfaces.iter_mut().find(|s| &s.layer == layer) else {
            return;
        };
        let (w, h) = configure.new_size;
        if w > 0 {
            s.logical.w = w;
        }
        if h > 0 {
            s.logical.h = h;
        }
        s.scale = s.bright.width as f64 / s.logical.w.max(1) as f64;
        match &s.viewport {
            Some(vp) => {
                vp.set_destination(s.logical.w as i32, s.logical.h as i32);
                s.layer.set_buffer_scale(1).ok();
            }
            None => {
                s.layer
                    .set_buffer_scale(s.scale.round().max(1.0) as u32)
                    .ok();
            }
        }
        s.configured = true;
        s.needs_draw = true;
    }
}

impl SeatHandler for App {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seats
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        match capability {
            Capability::Pointer if self.pointer.is_none() => {
                let surface = self.compositor.create_surface(qh);
                self.pointer = self
                    .seats
                    .get_pointer_with_theme::<_, ()>(
                        qh,
                        &seat,
                        self.shm.wl_shm(),
                        surface,
                        ThemeSpec::default(),
                    )
                    .ok();
            }
            Capability::Keyboard if self.keyboard.is_none() => {
                self.keyboard = self.seats.get_keyboard(qh, &seat, None).ok();
            }
            _ => {}
        }
    }

    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        match capability {
            Capability::Pointer => {
                if let Some(p) = self.pointer.take() {
                    p.pointer().release();
                }
            }
            Capability::Keyboard => {
                if let Some(k) = self.keyboard.take() {
                    k.release();
                }
            }
            _ => {}
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl PointerHandler for App {
    fn pointer_frame(
        &mut self,
        conn: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            let Some(surf) = self.surface(&event.surface) else {
                continue;
            };
            let (x, y) = event.position;
            let inside =
                x >= 0.0 && y >= 0.0 && x < surf.logical.w as f64 && y < surf.logical.h as f64;
            let pos = (surf.logical.x as f64 + x, surf.logical.y as f64 + y);

            match event.kind {
                PointerEventKind::Enter { .. } => {
                    self.cursor_icon = CursorIcon::ContextMenu;
                    if inside {
                        self.cursor = Some(pos);
                    }
                    self.mark_dirty();
                }
                PointerEventKind::Motion { .. } => {
                    self.cursor = Some(pos);
                    self.mark_dirty();
                }
                PointerEventKind::Leave { .. } => self.mark_dirty(),
                PointerEventKind::Press {
                    button: BTN_RIGHT, ..
                } => self.exit = true,
                PointerEventKind::Press { .. } => self.press(pos),
                PointerEventKind::Release { .. } => self.release(pos),
                _ => {}
            }
        }
        self.sync_cursor_icon(conn);
    }
}

impl KeyboardHandler for App {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _: &[Keysym],
    ) {
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: &wl_surface::WlSurface,
        _: u32,
    ) {
    }

    fn press_key(
        &mut self,
        conn: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        self.key(event.keysym);
        self.sync_cursor_icon(conn);
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        if matches!(event.keysym, Keysym::z | Keysym::Z) {
            self.zoom_held = false;
            self.mark_dirty();
        }
    }

    fn repeat_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _: KeyEvent,
    ) {
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        modifiers: Modifiers,
        _: RawModifiers,
        _: u32,
    ) {
        self.sel.set_shift(modifiers.shift);
        self.sel.set_ctrl(modifiers.ctrl);
    }
}

impl OutputHandler for App {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.outputs
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl ShmHandler for App {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for App {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }
    registry_handlers![OutputState, SeatState];
}

delegate_registry!(App);
smithay_client_toolkit::delegate_dispatch2!(App);
