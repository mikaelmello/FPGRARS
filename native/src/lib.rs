use neon::prelude::*;
use std::thread;

mod parser;
mod renderer;
mod simulator;

fn run(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let file = cx.argument::<JsString>(0)?.value();

    let sim = simulator::Simulator::new();
    let mmio = sim.memory.mmio.clone();

    thread::Builder::new()
        .name("FPGRARS Simulator".into())
        .spawn(move || {
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
            std::process::exit(0);
        })
        .expect("Could not run");

    renderer::init(mmio);

    Ok(cx.undefined())
}

register_module!(mut cx, {
    cx.export_function("run", run)?;

    Ok(())
});
