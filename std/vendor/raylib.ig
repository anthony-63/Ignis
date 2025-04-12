linkstatic "lib/vendor/libraylib.a"
linklib "gdi32"
linklib "winmm"

init_window -> extern[InitWindow](width i32, height i32, title string);
