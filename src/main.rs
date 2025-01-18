use glfw::{fail_on_errors, Action, Context, Key};

fn main() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();

    let (mut window, events) = glfw
        .create_window(800, 600, "Its Graphics Time", glfw::WindowMode::Windowed)
        .unwrap();

    window.set_all_polling(true);

    window.make_current();

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }

                _ => {}
                // uncomment to print 
                //e => { 
                    //println!("{:?}", e);
                //}
            }
        }
        window.swap_buffers();
    }
}
