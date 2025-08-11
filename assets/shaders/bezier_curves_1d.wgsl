// Bezier curve functions

fn bezier_curve3(a: f32, b: f32, c: f32, d: f32, t: f32) -> f32 {
  return (
    pow((1.0 - t), 3.0) * a +
    3.0 * pow((1.0 - t), 2.0) * t * b +
    3.0 * (1.0 - t) * pow(t, 2.0) * c +
    pow(t, 3.0) * d
  );
}

fn bezier_ease_in_out(a: f32, b: f32, t: f32) {
  return (
    pow((1.0 - t), 3.0) * a +
    3.0 * (1.0 - t) * pow(t, 2.0) * b +
    pow(t, 3.0)
  );
}
