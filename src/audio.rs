use rodio::Sink;
use std::fs::File;
use std::io::BufReader;

pub struct Audio {
    device: rodio::Device,
    sink: Option<Sink>,
}

impl Audio {
    pub fn new() -> Audio {
        let mut a = Audio {
            device: rodio::default_output_device().unwrap(),
            sink: None,
        };
        // nice, code that's safer in C++
        a.sink = Some(Sink::new(&a.device));

        return a;
    }

    pub fn play(&self) {
        let file = File::open("resources/out.mp3").unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

        self.sink.as_ref().unwrap().append(source);
        // this might just immediately play, we'll see
    }

    pub fn play_file(&self, filename: &String) {
        let file = match File::open(filename) {
            Ok(x) => x,
            Err(_) => return // this should print
        };
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

        self.sink.as_ref().unwrap().append(source);
        // this might just immediately play, we'll see
    }

    pub fn stop(&self) {
        self.sink.as_ref().unwrap().stop();
    }
}
