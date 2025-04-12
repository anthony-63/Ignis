include "std/io.ig"

Person -> struct {
    name string
}

main -> sub() {
    immut hello = new Person {
        name: "john",
    };

    writes(hello.name);
}