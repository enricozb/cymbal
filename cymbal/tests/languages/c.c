#include <stddef.h>
#include <stdint.h>

typedef unsigned int uint;
typedef struct Point Point;

struct Point {
  double x;
  double y;
};

typedef struct {
  float real;
  float imag;
} Complex;

union RawBytes {
  int32_t i;
  uint8_t bytes[4];
};

enum Direction {
  NORTH,
  SOUTH,
  EAST,
  WEST,
};

typedef enum Direction Direction;

double point_distance(Point a, Point b) {
  double dx = a.x - b.x;
  double dy = a.y - b.y;
  return __builtin_sqrt(dx * dx + dy * dy);
}

Complex complex_add(Complex a, Complex b) {
  return (Complex){ a.real + b.real, a.imag + b.imag };
}

int clamp(int value, int lo, int hi) {
  if (value < lo) return lo;
  if (value > hi) return hi;
  return value;
}
