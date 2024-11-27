# Sumatra

Sumatra is an (unfinished) JVM written in Rust. Sumatra targets Java SE 21 and endeavors to be fully compliant with the spec found [here](https://docs.oracle.com/javase/specs/jvms/se21/html/index.html).

Currently, Sumatra is a personal project and a personal challenge. While it is my goal to make Sumatra as production quality
as possible, it is not recommended to use in a production setting.

### Under Construction

The following are currently on the todo list:

- [x] Implement arrays
- [ ] Implement garbage collection
- [x] Implement a JNI
- [ ] Implement java.lang.* core classes' native methods
- [ ] Implement [Class file verification](https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.10)
- [ ] Implement multithreading support
- [ ] Implement I/O
- [ ] Implement JIT
- [ ] Implement Java Exceptions
- [ ] Implement MORE TESTS!
- [ ] Determine "[maximally specific method](https://docs.oracle.com/javase/specs/jls/se8/html/jls-15.html#jls-15.12.2.5)" during method resolution.
- [ ] Track how much memory is being used. 
