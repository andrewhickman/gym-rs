use derivative::Derivative;
use derive_new::new;
use sdl2::{
    event::Event,
    gfx::framerate::FPSManager,
    pixels::PixelFormatEnum,
    rect::{Point, Rect},
    render::WindowCanvas,
    EventPump,
};
use serde::Serialize;

use crate::utils::renderer::{RenderColor, RenderFrame, RenderMode, Renders};

/// Defines the structures required from SDL2 to process and render environments.
struct ScreenGui {
    pub canvas: WindowCanvas,
    pub fps_manager: FPSManager,
    pub event_pump: EventPump,
    // pub event_subsystem: EventSubsystem,
}

/// Defines a structure to encapsulate information about various transformations.
#[derive(new)]
pub struct ScreenGuiTransformations {
    src: Option<Rect>,
    dst: Option<Rect>,
    angle: f64,
    center: Option<Point>,
    flip_horizontal: bool,
    flip_vertical: bool,
}

impl ScreenGuiTransformations {
    /// Utility method to define a transformation which flips the GUI vertically.
    pub fn with_flip_vertical(self, flip_vertical: bool) -> Self {
        Self {
            flip_vertical,
            ..self
        }
    }
}

impl Default for ScreenGuiTransformations {
    fn default() -> Self {
        Self::new(None, None, 0., None, false, true)
    }
}

/// Defines a wrapper over SDL2, similar to PyGame to enable rapid development
/// of GUI environments.
#[derive(Serialize, Derivative, new)]
#[derivative(Debug)]
pub struct Screen {
    height: u32,
    width: u32,
    title: &'static str,
    render_fps: u32,
    mode: RenderMode,
    #[serde(skip_serializing)]
    #[derivative(Debug = "ignore")]
    #[new(default)]
    gui: Option<ScreenGui>,
}

impl Clone for Screen {
    fn clone(&self) -> Self {
        Self {
            height: self.height,
            width: self.width,
            title: self.title,
            render_fps: self.render_fps,
            mode: self.mode,
            gui: None,
        }
    }
}
impl Screen {
    /// Closes the process responsible for rendering the environment.
    pub fn close(&mut self) {
        self.gui.take();
    }

    /// Checks whether the screen is still available.
    pub fn is_open(&self) -> bool {
        self.gui.is_some()
    }

    /// Transforms the canvas into pixel coordinates for external consumption.
    fn canvas_to_pixels(canvas: &mut WindowCanvas, screen_width: u32) -> RenderFrame {
        let pixels = canvas
            .read_pixels(None, PixelFormatEnum::RGB24)
            .expect("pixels");

        let colours: Vec<RenderColor> = pixels
            .chunks(3)
            .map(|chunk| RenderColor::RGB(chunk[0], chunk[1], chunk[2]))
            .collect();

        let pixels_array: Vec<Vec<RenderColor>> = colours
            .chunks(screen_width as usize)
            .map(|chunk| chunk.into())
            .collect();

        RenderFrame::new(pixels_array)
    }

    /// Outputs the contents found in the GUI buffer to the display surface.
    pub fn render(&mut self, mode: RenderMode) -> Renders {
        match self.gui.as_mut() {
            Some(ScreenGui {
                canvas,
                fps_manager,
                ..
            }) => {
                fps_manager.delay();
                canvas.present();
                if [RenderMode::RgbArray, RenderMode::SingleRgbArray].contains(&mode) {
                    Renders::SingleRgbArray(Self::canvas_to_pixels(canvas, self.width))
                } else {
                    Renders::None
                }
            }
            _ => Renders::None,
        }
    }

    /// Outputs the width of the internal screen generated.
    pub fn screen_width(&self) -> u32 {
        self.width
    }

    /// Draws new content on the canvas using the closure and transformation instructions provided.
    pub fn draw_on_canvas(
        &mut self,
        using_fn: impl FnMut(&mut WindowCanvas),
        with_transformations: ScreenGuiTransformations,
    ) {
        if let Some(ScreenGui { canvas, .. }) = self.gui.as_mut() {
            let texture_creator = canvas.texture_creator();
            let mut texture = texture_creator
                .create_texture_target(PixelFormatEnum::RGB24, self.width, self.height)
                .expect("Create texture.");

            canvas
                .with_texture_canvas(&mut texture, using_fn)
                .expect("Was unable to render.");

            canvas
                .copy_ex(
                    &texture,
                    with_transformations.src,
                    with_transformations.dst,
                    with_transformations.angle,
                    with_transformations.center,
                    with_transformations.flip_horizontal,
                    with_transformations.flip_vertical,
                )
                .expect("Transformations failed to be applied.");
        }
    }

    /// Processes all events found in the queue.
    pub fn consume_events(&mut self) {
        if let Some(ScreenGui { event_pump, .. }) = self.gui.as_mut() {
            for event in event_pump.poll_iter() {
                if let Event::Quit { .. } = event {
                    panic!("Animation was forced to exit.")
                }
            }
        }
    }

    /// Generates a window to begin displaying content on.
    pub fn load_gui(&mut self) {
        if self.gui.is_none() {
            let title = self.title;
            let width = self.width;
            let height = self.height;
            let render_fps = self.render_fps;
            let mode = self.mode;

            let gui = {
                let context = sdl2::init().unwrap();
                let video_subsystem = context.video().unwrap();
                let mut window_builder = video_subsystem.window(title, width, height);

                window_builder.position_centered();

                if mode != RenderMode::Human {
                    window_builder.hidden();
                }

                let window = window_builder.build().unwrap();
                let canvas = window.into_canvas().accelerated().build().unwrap();
                let event_pump = context.event_pump().expect("Could not recieve event pump.");
                let mut fps_manager = FPSManager::new();
                // let event_subsystem = context
                //     .event()
                //     .expect("Event subsystem was not initialized.");
                fps_manager
                    .set_framerate(render_fps)
                    .expect("Framerate was unable to be set.");

                ScreenGui {
                    canvas,
                    event_pump,
                    // event_subsystem,
                    fps_manager,
                }
            };

            self.gui = Some(gui);
        }
    }
}
