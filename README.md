## Arl

Provides an iterator over console input events. There are two modes available:

### Symbol mode
Every keystroke is emitted as a separate event. Entered symbols aren't echoed to the console.

### Buffered line mode
The whole line entered is emitted as one event.

### Usage example

```rust 
let mut arl = Arl::new();
arl.line_mode();

for input in arl.start() {
    match input {
        Input::Line(query) => {
            // Start some asynchronous processing
            process_input(query);
            
            arl.symbol_mode(); 
        }
        Input::Symbol(Key::Esc) => {
            // abort processing, get back to the line mode 
            abort_input_processing();
            arl.line_mode();
        }
        Input::Symbol(Key::Char('q')) => {
            // execute 'q' command
            quit();
        }
        Input::Symbol(Key::Char('n')) => {
            // execute 'n' command
            next_page();
        }
        _ => ()
    }
}
```