
binary_name := "zneth"

cyan        := '\033[0;36m'
green       := '\033[0;32m'
yellow      := '\033[0;33m'
reset       := '\033[0m'

build:
    @echo "{{cyan}} Building compiler in release mode...{{reset}}"
    @cargo build --release --quiet
    
    @echo "{{cyan}} Recreating clean build space...{{reset}}"
    @rm -rf .build && mkdir -p .build
    
    @echo "{{cyan}} Isolate standalone executable...{{reset}}"
    @mv target/release/{{binary_name}} .build/
    
    @echo "\n{{green}} Success! Standalone binary ready at .build/{{binary_name}}{{reset}}"
