Person -> struct {
    a i32,
    test1 f32,

    add -> sub(b i32) i32 {
        return 1;
    }
}

main -> sub(args []str) void {
    immut i: f32 = 10;
    immut b = i + 10;
}