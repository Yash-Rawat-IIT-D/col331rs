# Initial process execs /init.
# This code runs in user space.

.globl start
start:
  add $1, %eax
  movl %eax, 0x10001
  jmp start