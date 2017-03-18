use engine::Canvas_;

pub struct OpenGLCanvas {
    
}

impl Canvas_ for OpenGLCanvas {
    fn print_info(&self) {
        println!("OpenGL, at the ready!");
    }
}
