use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use smithay_client_toolkit::dispatch2::Dispatch2;
use smithay_client_toolkit::output::{OutputHandler, OutputState};
use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
use smithay_client_toolkit::shm::slot::{Buffer, Slot, SlotPool};
use smithay_client_toolkit::shm::{Shm, ShmHandler};
use smithay_client_toolkit::{delegate_registry, registry_handlers};
use wayland_client::globals::registry_queue_init;
use wayland_client::protocol::{wl_output, wl_shm};
use wayland_client::{Connection, Proxy, QueueHandle, WEnum};
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_frame_v1::{
    self, ZwlrScreencopyFrameV1,
};
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1;

use crate::capture::{Capture, CursorLayer, Screen};
use crate::geom::{Rect, Transform};
use crate::raster::{Image, SourceFormat};

#[derive(Clone, Copy)]
struct Spec {
    format: wl_shm::Format,
    width: u32,
    height: u32,
    stride: u32,
}

enum Stage {
    AwaitingSpec,
    Copying { slot: Slot, _buffer: Buffer },
    Ready(Vec<u8>),
    Failed,
}

struct Frame {
    output: wl_output::WlOutput,
    with_cursor: bool,
    spec: Option<Spec>,
    y_invert: bool,
    stage: Stage,
}

struct FrameData;
struct ManagerData;

struct State {
    registry: RegistryState,
    outputs: OutputState,
    shm: Shm,
    pool: Option<SlotPool>,
    frames: HashMap<u32, Frame>,
}

pub fn capture_all(cursor: bool) -> Result<Capture> {
    let conn = Connection::connect_to_env().context("connect to wayland display")?;
    let (globals, mut queue) = registry_queue_init(&conn).context("registry init")?;
    let qh = queue.handle();

    let mut state = State {
        registry: RegistryState::new(&globals),
        outputs: OutputState::new(&globals, &qh),
        shm: Shm::bind(&globals, &qh).context("compositor has no wl_shm")?,
        pool: None,
        frames: HashMap::new(),
    };
    let manager: ZwlrScreencopyManagerV1 = globals
        .bind(&qh, 1..=3, ManagerData)
        .context("compositor does not support zwlr_screencopy_manager_v1")?;

    queue.roundtrip(&mut state).context("list outputs")?;
    queue
        .roundtrip(&mut state)
        .context("read output geometry")?;

    let outputs: Vec<_> = state.outputs.outputs().collect();
    if outputs.is_empty() {
        bail!("no outputs found");
    }
    let passes: &[bool] = if cursor { &[false, true] } else { &[false] };
    for output in &outputs {
        for &with_cursor in passes {
            let frame = manager.capture_output(with_cursor as i32, output, &qh, FrameData);
            state.frames.insert(
                frame.id().protocol_id(),
                Frame {
                    output: output.clone(),
                    with_cursor,
                    spec: None,
                    y_invert: false,
                    stage: Stage::AwaitingSpec,
                },
            );
        }
    }

    while state
        .frames
        .values()
        .any(|f| matches!(f.stage, Stage::AwaitingSpec | Stage::Copying { .. }))
    {
        queue
            .blocking_dispatch(&mut state)
            .context("dispatch capture")?;
    }

    let mut screens = Vec::new();
    let mut lit = Vec::new();
    for frame in std::mem::take(&mut state.frames).into_values() {
        let Stage::Ready(pixels) = frame.stage else {
            bail!("compositor rejected the screencopy request");
        };
        let info = state
            .outputs
            .info(&frame.output)
            .context("output vanished mid-capture")?;
        let spec = frame.spec.context("frame has no buffer spec")?;
        let format = match spec.format {
            wl_shm::Format::Xrgb8888 => SourceFormat::Bgrx,
            wl_shm::Format::Argb8888 => SourceFormat::Bgra,
            wl_shm::Format::Xbgr8888 => SourceFormat::Rgbx,
            wl_shm::Format::Abgr8888 => SourceFormat::Rgba,
            other => bail!("unsupported shm format {other:?}"),
        };
        let mut image = Image::from_raw(
            spec.width,
            spec.height,
            spec.stride as usize,
            format,
            &pixels,
        )?;
        if frame.y_invert {
            image = image.flipped_vertically();
        }
        let image = image.transformed(transform(info.transform));

        let (x, y) = info.logical_position.unwrap_or((0, 0));
        let (w, h) = info
            .logical_size
            .map_or((image.width, image.height), |(w, h)| (w as u32, h as u32));
        let image = Arc::new(image);
        if frame.with_cursor {
            lit.push((frame.output, image));
            continue;
        }
        screens.push((
            frame.output,
            Screen {
                name: info.name.clone().unwrap_or_default(),
                logical: Rect::new(x, y, w, h),
                scale: image.width as f64 / w.max(1) as f64,
                image,
                cursor: None,
            },
        ));
    }

    for (output, image) in lit {
        if let Some((_, screen)) = screens.iter_mut().find(|(o, _)| *o == output) {
            screen.cursor = CursorLayer::between(&screen.image, &image);
        }
    }
    Ok(Capture::new(screens.into_iter().map(|(_, s)| s).collect()))
}

fn transform(t: wl_output::Transform) -> Transform {
    match t {
        wl_output::Transform::_90 => Transform::Rotate90,
        wl_output::Transform::_180 => Transform::Rotate180,
        wl_output::Transform::_270 => Transform::Rotate270,
        wl_output::Transform::Flipped => Transform::Flipped,
        wl_output::Transform::Flipped90 => Transform::FlippedRotate90,
        wl_output::Transform::Flipped180 => Transform::FlippedRotate180,
        wl_output::Transform::Flipped270 => Transform::FlippedRotate270,
        _ => Transform::Normal,
    }
}

impl State {
    fn start_copy(&mut self, frame: &ZwlrScreencopyFrameV1, key: u32) -> Result<()> {
        let spec = self
            .frames
            .get(&key)
            .and_then(|f| f.spec)
            .context("compositor advertised no shm format")?;
        let len = spec.stride as usize * spec.height as usize;
        if len == 0 {
            bail!("compositor reported a zero-sized frame");
        }
        let pool = match &mut self.pool {
            Some(pool) => pool,
            None => self
                .pool
                .insert(SlotPool::new(len, &self.shm).context("create shm pool")?),
        };
        let slot = pool.new_slot(len).context("allocate shm slot")?;
        let buffer = pool
            .create_buffer_in(
                &slot,
                spec.width as i32,
                spec.height as i32,
                spec.stride as i32,
                spec.format,
            )
            .context("create wl_buffer")?;
        frame.copy(buffer.wl_buffer());
        if let Some(f) = self.frames.get_mut(&key) {
            f.stage = Stage::Copying {
                slot,
                _buffer: buffer,
            };
        }
        Ok(())
    }

    fn finish_copy(&mut self, key: u32) {
        let Some(f) = self.frames.get_mut(&key) else {
            return;
        };
        let pixels = match (&f.stage, f.spec, &mut self.pool) {
            (Stage::Copying { slot, .. }, Some(spec), Some(pool)) => {
                let len = spec.stride as usize * spec.height as usize;
                pool.raw_data_mut(slot).get(..len).map(<[u8]>::to_vec)
            }
            _ => None,
        };
        f.stage = pixels.map_or(Stage::Failed, Stage::Ready);
    }

    fn fail(&mut self, key: u32) {
        if let Some(f) = self.frames.get_mut(&key) {
            f.stage = Stage::Failed;
        }
    }
}

impl Dispatch2<ZwlrScreencopyFrameV1, State> for FrameData {
    fn event(
        &self,
        state: &mut State,
        frame: &ZwlrScreencopyFrameV1,
        event: zwlr_screencopy_frame_v1::Event,
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
        use zwlr_screencopy_frame_v1::Event;

        let key = frame.id().protocol_id();
        match event {
            Event::Buffer {
                format: WEnum::Value(format),
                width,
                height,
                stride,
            } => {
                if let Some(f) = state.frames.get_mut(&key) {
                    f.spec.get_or_insert(Spec {
                        format,
                        width,
                        height,
                        stride,
                    });
                }
            }
            Event::Flags {
                flags: WEnum::Value(flags),
            } => {
                if let Some(f) = state.frames.get_mut(&key) {
                    f.y_invert = flags.contains(zwlr_screencopy_frame_v1::Flags::YInvert);
                }
            }
            Event::BufferDone => {
                if let Err(err) = state.start_copy(frame, key) {
                    eprintln!("{}: screencopy failed: {err:#}", crate::app::NAME);
                    state.fail(key);
                }
            }
            Event::Ready { .. } => {
                state.finish_copy(key);
                frame.destroy();
            }
            Event::Failed => {
                state.fail(key);
                frame.destroy();
            }
            _ => {}
        }
    }
}

impl Dispatch2<ZwlrScreencopyManagerV1, State> for ManagerData {
    fn event(
        &self,
        _: &mut State,
        _: &ZwlrScreencopyManagerV1,
        _: <ZwlrScreencopyManagerV1 as Proxy>::Event,
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
    }
}

impl OutputHandler for State {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.outputs
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl ShmHandler for State {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }
    registry_handlers![OutputState];
}

delegate_registry!(State);
smithay_client_toolkit::delegate_dispatch2!(State);
