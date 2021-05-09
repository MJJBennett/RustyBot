use rodio::Sink;
use std::fs::File;
use std::io::BufReader;
use rodio::Source;

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
}
