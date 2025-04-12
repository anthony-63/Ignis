include "std/vendor/raylib.ig"
include "std/io.ig"

main -> sub() {
    init_window(800, 600, "Hello world!");

    let red = new Color{r: 255, g: 0, b: 0, a: 255};
    let i = 0; 

    while !(window_should_close()) {
        begin_drawing();
        clear_background(red);
        i = i + 1;
        end_drawing();
    }
}
