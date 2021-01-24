use neon::prelude::*;
use neon::{declare_types, register_module};
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

mod parser;
mod renderer;
mod simulator;

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 240;
pub const FRAME_SELECT: usize = 0x20_0604;
pub const FRAME_0: usize = 0;
pub const FRAME_1: usize = 0x10_0000;

pub struct State {
  simulator: Arc<RwLock<simulator::Simulator>>,
}

declare_types! {
  pub class JsState for State {
    init(mut cx) {

      Ok(State {
        simulator: Arc::new(RwLock::new(simulator::Simulator::new())),
      })
    }

    method run(mut cx) {
        let file = cx.argument::<JsString>(0)?.value();
        let this = cx.this();
        let guard = cx.lock();
        let state = this.borrow(&guard);
        let simulator = state.simulator.clone();

        if let Ok(mut write_guard) = simulator.write() {
            *write_guard = simulator::Simulator::new();
        }

        thread::Builder::new()
            .name("FPGRARS Simulator".into())
            .spawn(move || {

            // read() will only block when `producer_thread` is holding a write lock
            if let Ok(read_guard) = simulator.read() {
              let sim = &*read_guard;
                let mut sim = match sim.load_from_file(file) {
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("An error occurred while parsing your code:\n{:?}", e);
                        std::process::exit(0);
                    }
                };
                let start_time = std::time::Instant::now();
                sim.run();
                println!("Finished in {}ms", start_time.elapsed().as_millis());
                panic!("finished");
              }
            })
            .expect("Could not run");

        Ok(cx.undefined().upcast())
    }

    method get_mmio(mut cx) {
        let length = WIDTH * HEIGHT;
        let mut buffer = JsArrayBuffer::new(&mut cx, length as u32)?;

        let this = cx.this();
        let guard = cx.lock();
        let state = this.borrow(&guard);
        let simulator = state.simulator.clone();
        // read() will only block when `producer_thread` is holding a write lock
        if let Ok(read_guard) = simulator.read() {
          let simulator = &*read_guard;
        let mmio = simulator.memory.mmio.clone();
        let mmio = mmio.lock().unwrap();

        let frame = mmio[FRAME_SELECT];
        let start = if frame == 0 { FRAME_0 } else { FRAME_1 };

        let mmio_slice = &mmio[start..start+length];
        cx.borrow_mut(&mut buffer, |slice| {
            let slice = slice.as_mut_slice::<u8>();
            slice.clone_from_slice(mmio_slice);
        });
      }

        Ok(buffer.upcast())
    }

    method panic(_) {
      panic!("State.prototype.panic")
    }
  }
}

register_module!(mut m, { m.export_class::<JsState>("State") });
