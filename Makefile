all:
	cd os && make

fat32:
	cd os && make start-fat32

ext4:
	cd os && make start-ext4

gdb:
	cd os && make gdb

clean:
	cd os && make clean