include "std/vendor/raylib.ig"
include "std/io.ig"

main -> sub() {
    init_window(800, 600, "Hello world!");

    while !(window_should_close()) {
        begin_drawing();

        end_drawing();
    }
}
