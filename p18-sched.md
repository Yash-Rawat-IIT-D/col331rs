## Scheduler

In `p17`, we started two processes, but only one kept running.  
This is starvation. In `p18`, the OS switches between runnable processes.

Changes in this part:
- Re-enable LAPIC timer interrupts.
- Add `yield` in the trap path for timer interrupts.
- Implement two-way context switching via `swtch(old, new)`.

Flow:
1. `scheduler` picks a `RUNNABLE` process.
2. `swtch(&c.scheduler, p.context)` saves scheduler context and starts process context.
3. Timer interrupt arrives while process is running.
4. `trap` calls `yield`, which marks process `RUNNABLE` and calls `sched`.
5. `sched` does `swtch(&p.context, c.scheduler)` and returns to scheduler loop.
6. Scheduler picks next runnable process.

This gives round-robin scheduling between active processes.
