linkstatic "lib/vendor/libraylib.a"
linklib "gdi32"
linklib "winmm"

Color -> struct {
    r i8,
    g i8,
    b i8,
    a i8
}

init_window -> extern[InitWindow](width i32, height i32, title string);
window_should_close -> extern[WindowShouldClose]() bool;

begin_drawing -> extern[BeginDrawing]();

clear_background -> extern[ClearBackground](color Color);

end_drawing -> extern[EndDrawing]();