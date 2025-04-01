Point -> struct {
    x int, 
    y int,

    distance -> sub(this, p2 Point) float64 {
        mut dx = p2.x - this.x;
        mut dy = p2.y - this.y;
        return sqrt(dx*dx + dy*dy);
    }

    move -> sub(&this, dx int, dy int) void {
        this.x += dx;
        this.y += dy;
    }
}

main -> sub() void {
    mut p1 = Point{x: 0, y: 0};
    mut p2 = Point{x: 3, y: 4};

    mut result = p1.distance(p2);
    log("Distance:", result);

    p1.move(5, 7);
    log("Moved p1 to:", p1.x, p1.y);

    immut constant_value = 10;
}

sqrt -> sub(x float64) float64 {
    return x ** 0;
}