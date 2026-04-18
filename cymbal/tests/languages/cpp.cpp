#include <string>
#include <vector>

typedef unsigned int uint;

struct Vec2 {
  double x;
  double y;
};

class Shape {
public:
  virtual ~Shape() = default;
  virtual double area() const = 0;
  virtual std::string name() const = 0;
};

class Circle : public Shape {
public:
  explicit Circle(double radius) : radius_(radius) {}
  double area() const override;
  std::string name() const override;

private:
  double radius_;
};

double Circle::area() const {
  return 3.14159265358979 * radius_ * radius_;
}

std::string Circle::name() const {
  return "Circle";
}

class Rectangle : public Shape {
public:
  Rectangle(double w, double h) : width_(w), height_(h) {}
  double area() const override { return width_ * height_; }
  std::string name() const override { return "Rectangle"; }

private:
  double width_, height_;
};

double total_area(const std::vector<Shape*>& shapes) {
  double sum = 0.0;
  for (auto* s : shapes) sum += s->area();
  return sum;
}

class Polygon : public Shape {
public:
  struct Edge {
    Vec2 start;
    Vec2 end;

    double length() const;
    Vec2 midpoint() const;
  };

  explicit Polygon(std::vector<Edge> edges) : edges_(std::move(edges)) {}
  double area() const override;
  std::string name() const override;

private:
  std::vector<Edge> edges_;
};

double Polygon::Edge::length() const {
  double dx = end.x - start.x;
  double dy = end.y - start.y;
  return dx * dx + dy * dy;
}

Vec2 Polygon::Edge::midpoint() const {
  return {(start.x + end.x) / 2.0, (start.y + end.y) / 2.0};
}

double Polygon::area() const {
  return 0.0;
}

std::string Polygon::name() const {
  return "Polygon";
}
