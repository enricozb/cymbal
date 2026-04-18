package main

import "core:fmt"
import math "core:math"

Vec2 :: struct {
  x: f64,
  y: f64,
}

Color :: enum {
  Red,
  Green,
  Blue,
}

add :: proc(a, b: f64) -> f64 {
  return a + b
}

vec2_length :: proc(v: Vec2) -> f64 {
  return math.sqrt(v.x * v.x + v.y * v.y)
}

vec2_add :: proc(a, b: Vec2) -> Vec2 {
  return Vec2{a.x + b.x, a.y + b.y}
}

main :: proc() {
  v := Vec2{3.0, 4.0}
  fmt.println(vec2_length(v))
}
