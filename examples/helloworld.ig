include "std/io.ig"

Person -> struct {
    name string
}

main -> sub() {
    let hello = new Person {
        name: "john",
    };

    writes(hello.name);
}