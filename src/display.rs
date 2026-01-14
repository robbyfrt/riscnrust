use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
    primitives::{Rectangle, PrimitiveStyle},
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
}

fn wrap_text(text: &str, display_size: (usize, usize), font_size: (usize, usize)) -> Vec<&str> {
    let display_width_chars = display_size.0 / font_size.0;
    text.as_bytes()
        .chunks(display_width_chars)
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect()
}
