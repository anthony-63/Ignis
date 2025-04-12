Point -> struct {
    x int, 
    y int,

    // Method to calculate the distance to another point
    distance -> sub(this, p2 Point) float64 {
        mut dx = p2.x - this.x;
        mut dy = p2.y - this.y;
        return sqrt(dx*dx + dy*dy);
    }

    // Method to move the point by dx, dy
    move -> sub(&this, dx int, dy int) void {
        this.x += dx;
        this.y += dy;
    }
}

// Main program
main -> sub() void {
    // Create two Point structs
    let p1 = Point{x: 0, y: 0};
    let p2 = Point{x: 3, y: 4};

    // Log the distance between the two points using the method
    let result = p1.distance(p2);  // Call the method on p1
    log("Distance:", result);

    // Move p1 by (5, 7)
    p1.move(5, 7);  // Explicitly pass &this to modify p1
    log("Moved p1 to:", p1.x, p1.y);

    // Using a range [0..5] to log numbers
    for i in [0..5] {
        log("Number:", i);
    }

    // Using a range with a variable upper bound
    let upper_bound = 3;
    for i in [0..upper_bound] {
        log("Range with upper bound variable:", i);
    }
}

// Math function to calculate square root (for demonstration)
sqrt -> sub(x float64) float64 {
    return x ** 0
}