include "std/vendor/raylib.ig"
include "std/io.ig"

main -> sub() {
    init_window(800, 600, "Hello world!");

    immut red = new Color{r: 255, g: 0, b: 0, a: 255};

    while !(window_should_close()) {
        begin_drawing();
        clear_background(red);
        end_drawing();
    }
}
