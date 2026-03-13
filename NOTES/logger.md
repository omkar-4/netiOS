# Building the Kernel Logger

## External Crates
```Cargo.toml
[dependencies]
noto-sans-mono-bitmap = "0.3"
```

## MUTEX (Mutual Exclusion Object)
A mutex (mutual exclusion object) is a synchronization mechanism used in computer programming to manage access to a shared resource, ensuring only one thread or process uses it at a time.

It acts as a lock, preventing race conditions and ensuring data integrity by allowing only the holding thread to unlock it.

`Mutually exclusive` means 2 or more events/outcomes/choices cannot happen/exist simultanously. If one occurs, other is impossible.

## cli & sti (clean interrupt and set interrupt flags)

##

Replaced this code from the end of _main fn in main.rs:
- It was to test if the interrupt is working (before keyboard interupt was implemented), which wasn't very useful as it didn't give any visible output, so it's shit code anyways.
```rs
    println!("IDT Loaded. Triggering hardware interrupt...");

    // ring alarm manually
    unsafe {
        core::arch::asm!("int3");
    }

    println!("Successfully returned from interrupt!");
```

- replace this:
```rs
fn write_char(&mut self, c: char) {
if c == '\n' {
    self.x = 10;
    self.y += 24;
    return;
}

if self.x + raster.width() > self.width {
    self.x = 10;
    self.y += 24;
}
```
- with:
```rs
if c == '\n' {
    self.newline();
    return;
}

if self.x + raster.width() > self.width {
    self.newline();
}
```
