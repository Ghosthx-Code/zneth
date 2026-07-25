# Zneth Coding Lang

> [!WARNING]
> I Have Stopped working on this lang
> i do not think it will be good to so i am making a new lang

> [!WARNING]
> This Lang Can only Print, Make Vars, Function ( without params, or calling ), no math
> Prining Can Only Do Int/String's, no float but i will impl that
> Do not use the lang for anything big it is just for fun right know 

> [!NOTE]
> You have to have rust/justfile/llvm/clang installed

### To Install the JustFile/Clang
```bash
sudo apt install just clang
```
```bash
sudo dnf install just clang
```
```bash
sudo pacman -S just clang
```

## For people wanting to use this compile
###### i will go over some syntax

### To make main function or any function it is
```c
signed static fn main() ?i32 {  }
```
signed says it will be allocad to the stack, ?i32 says it will have a return value all functions do

### Printing
###### all syntax will be in the main function, but i will not show the main function
```c
printf("Hello world");
printf(13);
printf("Hello zneth!");
```

### Vars
```c
signed x str = "hello world";
unsigned y str = "Hello world";
```
unsigned mean it is allocad to the heap, signed is the stack

### Return
```c
ret 0;
```

### Data Types
```c
i32, i64, i128
f32, f64
i8, i1
str
```

### Installing
```bash
cd ~
git clone https://github.com/Ghosthx-Code/zneth
cd zneth
just build
mv .build/zneth .
sudo cp zneth /usr/local/bin/zneth
zneth --help
```

### Some stuff about the lang
wen you compile the exe will be in the `./target/`
all exe are static linked

### Commands ( that work )
```
new : makes a new project folder
build : builds the project folder
switch : wen you run build it makes a copy of the src/main.z and puts it in ./target/Debug/{version_num}/debug.z, with switch you can switch from one file to the other file, tip use -cc to see all the version's
--help : displays help menu
```
