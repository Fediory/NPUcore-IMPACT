case $1 in
    "easy-fs") mv ../easy-fs/target/doc ../Doc/fs/doc;;
    "os") rm -rf ../Doc/os/doc;mv target/riscv64gc-unknown-none-elf/doc ../Doc/os/doc;;
esac
