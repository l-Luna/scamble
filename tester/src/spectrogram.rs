use vizia::prelude::*;
use vizia::vg::*;

type Spectrum = [f32; 1024];

pub struct Spectrogram<L> {
    lens: L,
    colour: vizia::prelude::Color,
}

impl<L: Lens<Target = Spectrum>> Spectrogram<L> {
    pub fn new(
        cx: &mut Context,
        colour: vizia::prelude::Color,
        lens: L,
    ) -> vizia::prelude::Handle<Self> {
        Self { lens, colour }.build(cx, move |cx| {})
    }
}

impl<L: Lens<Target = Spectrum>> View for Spectrogram<L> {
    fn element(&self) -> Option<&'static str> {
        Some("spectrogram")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let opacity = cx.opacity();
        let foreground_colour = cx.font_color();
        let bounds = cx.bounds();

        let mut points: [Point; 512] = [Point::new(0., 0.); 512];
        let spectra = self.lens.get(cx);
        for i in 0..512 {
            points[i] = Point::new(
                bounds.left() + bounds.w * (i as f32 / 511.),
                bounds.bottom() - bounds.h * (spectra[i * 2].sqrt().min(1.)),
            );
        }

        let mut shape = [Point::new(0., 0.); 514];
        shape[..512].copy_from_slice(&points);
        shape[512] = Point::from(bounds.bottom_right());
        shape[513] = Point::from(bounds.bottom_left());

        let mut path = Path::new();
        path.add_poly(&points, false);

        let mut paint = Paint::default();
        paint.set_color(foreground_colour);
        paint.set_style(PaintStyle::Stroke);
        paint.set_stroke_width(1.5);
        paint.set_alpha_f(opacity);
        canvas.draw_path(&path, &paint);

        let mut path = Path::new();
        path.add_poly(&shape, true);
        paint.set_style(PaintStyle::Fill);
        paint.set_alpha_f(opacity / 4.);
        paint.set_color(self.colour);
        canvas.draw_path(&path, &paint);
    }
}
