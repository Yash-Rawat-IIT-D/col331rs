# Initial process that runs in user space.

#include "syscall.h"
#include "traps.h"

.globl start
start:

# sys_open("console", O_WRONLY)
pushl $1
pushl $console
pushl $0
movl $SYS_open, %eax
int $T_SYSCALL

# sys_write(fd, "hello world\n\0")
pushl $hello
pushl %eax
pushl $0
movl $SYS_write, %eax
int $T_SYSCALL

movl $SYS_close, %eax
int $T_SYSCALL

  jmp start

console:
  .string "console\0"

hello:
  .string "hello world from inside the process!\n\0"
