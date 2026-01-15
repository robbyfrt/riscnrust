use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
    primitives::{Rectangle, PrimitiveStyle, Line},
};
use ssd1306::{prelude::*, Ssd1306};
use log::info;

pub struct DisplayManager<DI> {
    display: Ssd1306<DI, DisplaySize128x64, ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>>,
    text_style: MonoTextStyle<'static, BinaryColor>,
}

impl<DI> DisplayManager<DI>
where
    DI: ssd1306::prelude::WriteOnlyDataCommand,
{
    pub fn new(interface: DI, rotation: DisplayRotation) -> anyhow::Result<Self> {
        let mut display = Ssd1306::new(
            interface,
            DisplaySize128x64,
            rotation,
        )
        .into_buffered_graphics_mode();
        
        display.init().map_err(|_| anyhow::anyhow!("Display init failed"))?;

        let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

        Ok(Self { display, text_style })
    }

    pub fn clear(&mut self) -> anyhow::Result<()> {
        self.display.clear(BinaryColor::Off)
            .map_err(|_| anyhow::anyhow!("Clear failed"))
    }
    
    pub fn log_and_show(&mut self, message: &str) -> anyhow::Result<()> {
        info!("{}", message);
        self.clear()?;
        
        for (i, line) in wrap_text(message, (128, 64), (6, 10)).iter().enumerate() {
            Text::with_baseline(
                line,
                Point::new(0, (i * 10) as i32),
                self.text_style,
                Baseline::Top,
            )
            .draw(&mut self.display)
            .map_err(|_| anyhow::anyhow!("Draw error"))?;
        }
        
        self.flush()
    }

    pub fn update_line(&mut self, line_number: usize, message: &str) -> anyhow::Result<()> {
        Rectangle::new(
            Point::new(0, (line_number * 10) as i32),
            Size::new(128, 10),
        )
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
        .draw(&mut self.display)
        .map_err(|_| anyhow::anyhow!("Rectangle draw error"))?;

        Text::with_baseline(
            message,
            Point::new(0, (line_number * 10) as i32),
            self.text_style,
            Baseline::Top,
        )
        .draw(&mut self.display)
        .map_err(|_| anyhow::anyhow!("Text draw error"))?;
        
        Ok(())
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        self.display.flush()
            .map_err(|_| anyhow::anyhow!("Flush failed"))
    }
    pub fn draw_rect(&mut self, top_left: (u32,u32), bottom_right: (u32,u32)) -> anyhow::Result<()> {
        let top_left = Point { x: top_left.0 as i32, y: top_left.1 as i32};
        let bottom_right = Point { x: bottom_right.0 as i32, y: bottom_right.1 as i32};
        Rectangle::new(top_left, Size::new((bottom_right.x - top_left.x) as u32, (bottom_right.y - top_left.y) as u32))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut self.display)
            .map_err(|_| anyhow::anyhow!("Rectangle draw error"))
    }
    #[allow(dead_code)]
    fn scale_xy_to_raph_coords(&self, points: &[(i32, i32)], graph_area: (i32, i32, i32, i32)) -> Vec<(i32, i32)> {
        let (x_min, y_min) = points.iter().fold((i32::MAX, i32::MAX), |(min_x, min_y), &(x, y)| {
            (min_x.min(x), min_y.min(y))
        });
        let (x_max, y_max) = points.iter().fold((i32::MIN, i32::MIN), |(max_x, max_y), &(x, y)| {
            (max_x.max(x), max_y.max(y))
        });

        let (graph_x_min, graph_y_min, graph_x_max, graph_y_max) = graph_area;
        let graph_width = graph_x_max - graph_x_min;
        let graph_height = graph_y_max - graph_y_min;

        points.iter().map(|&(x, y)| {
            let scaled_x = if x_max != x_min {
                graph_x_min + ((x - x_min) * graph_width) / (x_max - x_min)
            } else {
                graph_x_min + graph_width / 2
            };
            let scaled_y = if y_max != y_min {
                graph_y_max - ((y - y_min) * graph_height) / (y_max - y_min)
            } else {
                graph_y_min + graph_height / 2
            };
            (scaled_x, scaled_y)
        }).collect()
    }
    #[allow(dead_code)]
    pub fn draw_xy_graph(&mut self, points: &[(i32, i32)]) -> anyhow::Result<()> {
        let scaled_points = self.scale_xy_to_raph_coords(points, (0, 0, 128, 64));
        for window in scaled_points.windows(2) {
            let start = Point::new(window[0].0, window[0].1);
            let end = Point::new(window[1].0, window[1].1);
            Line::new(start, end)
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                .draw(&mut self.display)
                .map_err(|_| anyhow::anyhow!("Line draw error"))?;
        }
        Ok(())
    }
}

fn wrap_text(text: &str, display_size: (usize, usize), font_size: (usize, usize)) -> Vec<&str> {
    let display_width_chars = display_size.0 / font_size.0;
    text.as_bytes()
        .chunks(display_width_chars)
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect()
}
