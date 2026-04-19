use std::env;
use std::process;

use luwi_script::driver;

/// Entry point do compilador LuwiScript (`luwic`).
#[allow(clippy::needless_return)] // Só para ficar claro.
fn main() {
    let args: Vec<String> = env::args().collect();

    let status = driver::run(args);

    if status.is_err() {
        process::exit(1);
    }
}
