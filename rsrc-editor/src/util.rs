use eframe::egui::{
    Ui, Widget, Rect, Scene, Vec2, Sense, Pos2, CornerRadius, Color32,
};
use macfmt::rsrc::types::Icon;

pub fn icon_editor<const SIZE: usize>(icon: &mut Icon<SIZE>, rect: &mut Rect) -> impl Widget {
    move |ui: &mut Ui| {
        Scene::new()
            .zoom_range(0.0..=100.0)
            .show(ui, rect, |ui| {
                let size = Vec2 {
                    x: icon.side() as f32,
                    y: icon.side() as f32,
                };

                let (resp, painter) = ui.allocate_painter(
                    size,
                    Sense::HOVER | Sense::CLICK,
                );
                if resp.clicked()
                    && let Some(Pos2 { x, y }) = resp.interact_pointer_pos()
                {
                    let x = x as usize;
                    let y = y as usize;
                    let val = icon.pixel(x, y);
                    icon.set_pixel(x, y, !val);
                }
                let mut img = icon.image();

                for (x, y, px) in img.enumerate_pixels_mut() {
                    let pos = Pos2 {
                        x: (x as f32),
                        y: (y as f32),
                    };
                    let pixel = Vec2 { x: 1.0, y: 1.0 };
                    let rect = Rect::from_min_size(pos, pixel);
                    painter.rect_filled(
                        rect,
                        CornerRadius::ZERO,
                        Color32::from_gray(!px[0]),
                    );
                }
            }).response
    }
}
