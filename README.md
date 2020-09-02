# mouse\_sovec

`mouse\_sovec` defines `SoVec` for `mouse`, which behaves like `std::collections::Vec` .

(`SoVec` stands for `Small optimized Vector` .)

`SoVec` will not allocate heap if the requested memory size is small enough. 
Instead of heap, it uses itself as a buffer then.

To avoid allocating as much as possible, the performance is better than that of `std::collections::Vec` .
