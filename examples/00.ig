linklib "c"

puts -> extern(str string) i32;

print -> sub(str string) {
    puts(str);
}

main -> sub() {
    print("hi");
}