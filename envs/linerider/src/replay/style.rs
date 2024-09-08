use egui::{Style, FontDefinitions, Context, TextStyle, FontFamily, Vec2};

pub fn customize_style(ctx: &Context) {
  // let style = get_custom_style();
  // ctx.set_style(style);
  // let mut fonts = FontDefinitions::default();

  // TODO: Increase Font Size everywhere
  // ctx.set_fonts(fonts);
}

pub fn get_custom_style() -> Style {
  let mut style = Style::default();
  style.spacing.button_padding = Vec2::new(6.0, 6.0);
  style.spacing.scroll_bar_width = 10.0;
  style.spacing.icon_width = 18.0;
  // style.spacing.window_margin = Vec2::new(0.0, 0.0);
  style
}