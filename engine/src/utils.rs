use std::{thread::sleep, time::Duration};

#[macro_export]
macro_rules! exit {
    ($msg:expr, $($a:ident),*) => {
        $crate::utils::_exit(format!($msg, $($a),*).as_str());
        panic!();
    };

    ($msg:expr) => {
        $crate::utils::_exit(format!($msg).as_str());
        panic!();
    };
}

pub fn _exit(msg: &str) {
    println!(
        "########## A critical error occured! The program will close in 15 seconds. ##########"
    );
    println!("Error message: {msg}");
    sleep(Duration::new(15, 0));
}

#[macro_export]
macro_rules! get_gl_error {
    () => {    let a = unsafe { gl::GetError() };
        match a {
            0 => {} // no user error
            1280 => println!("GL ERROR({}, {}): Invalid enum param", file!(), line!()),
            1281 => println!("GL ERROR({}, {}): Invalid value param", file!(), line!()),
            1282 => println!(
                "GL ERROR({}, {}): Invalid operation, set when the state of a command is not legal for its given params",
                file!(),
                line!()
            ),
            1283 => println!("GL ERROR({}, {}): Stack overflow", file!(), line!()),
            1284 => println!("GL ERROR({}, {}): Stack underflow", file!(), line!()),
            1285 => println!("GL ERROR({}, {}): Out of memory", file!(), line!()),
            1286 => println!(
                "GL ERROR({}, {}): Invalid framebuffer operation, set when reading or writing to a framebuffer that is not complete",
                file!(),
                line!()
            ),
            _ => {}
        }
    }
}
