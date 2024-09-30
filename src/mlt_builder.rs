use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::fs::File;

pub struct MltBuilder {
    timestamps: Vec<(f64, f64)>,
    duration: f64,
    input_file: String,
    output_file: String,
}

impl MltBuilder {
    pub fn new() -> Self {
        MltBuilder {
            timestamps: Vec::new(),
            duration: 0.0,
            input_file: String::new(),
            output_file: String::new(),
        }
    }

    pub fn timestamps(mut self, timestamps: Vec<(f64, f64)>) -> Self {
        self.timestamps = timestamps;
        self
    }

    pub fn duration(mut self, duration: f64) -> Self {
        self.duration = duration;
        self
    }

    pub fn input_file(mut self, input_file: &str) -> Self {
        self.input_file = input_file.to_string();
        self
    }

    pub fn output_file(mut self, output_file: &str) -> Self {
        self.output_file = output_file.to_string();
        self
    }

    pub fn build(self) {
        let total_duration = format_time(self.duration);

        let mut writer = Writer::new_with_indent(File::create(&self.output_file).unwrap(), b' ', 4);

        writer
            .write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"utf-8"), None)))
            .unwrap();

        let mut mlt = BytesStart::borrowed_name(b"mlt");
        mlt.push_attribute(("LC_NUMERIC", "C"));
        mlt.push_attribute(("version", "7.27.0"));
        mlt.push_attribute(("producer", "main_bin"));
        writer.write_event(Event::Start(mlt)).unwrap();

        let mut profile = BytesStart::borrowed_name(b"profile");
        profile.push_attribute(("width", "2560"));
        profile.push_attribute(("height", "1440"));
        profile.push_attribute(("progressive", "1"));
        profile.push_attribute(("sample_aspect_num", "1"));
        profile.push_attribute(("sample_aspect_den", "1"));
        profile.push_attribute(("display_aspect_num", "16"));
        profile.push_attribute(("display_aspect_den", "9"));
        profile.push_attribute(("frame_rate_num", "60000000"));
        profile.push_attribute(("frame_rate_den", "1000000"));
        profile.push_attribute(("colorspace", "709"));
        writer.write_event(Event::Empty(profile)).unwrap();

        let mut playlist = BytesStart::borrowed_name(b"playlist");
        playlist.push_attribute(("id", "main_bin"));
        writer.write_event(Event::Start(playlist)).unwrap();
        writer.write_event(Event::End(BytesEnd::borrowed(b"playlist"))).unwrap();

        let mut producer = BytesStart::borrowed_name(b"producer");
        producer.push_attribute(("id", "black"));
        producer.push_attribute(("in", "00:00:00.000"));
        producer.push_attribute(("out", &total_duration[..]));
        writer.write_event(Event::Empty(producer)).unwrap();

        let mut background_playlist = BytesStart::borrowed_name(b"playlist");
        background_playlist.push_attribute(("id", "background"));
        writer.write_event(Event::Start(background_playlist)).unwrap();
        
        let mut entry = BytesStart::borrowed_name(b"entry");
        entry.push_attribute(("producer", "black"));
        entry.push_attribute(("in", "00:00:00.000"));
        entry.push_attribute(("out", &total_duration[..]));
        writer.write_event(Event::Empty(entry)).unwrap();
        writer.write_event(Event::End(BytesEnd::borrowed(b"playlist"))).unwrap();

        for (i, _) in self.timestamps.iter().enumerate() {
            let mut chain = BytesStart::borrowed_name(b"chain");
            chain.push_attribute(("id", &format!("chain{}", i)[..]));
            chain.push_attribute(("out", &total_duration[..]));
            writer.write_event(Event::Start(chain)).unwrap();

            let mut property = BytesStart::borrowed_name(b"property");
            property.push_attribute(("name", "resource"));
            writer.write_event(Event::Start(property)).unwrap();
            writer
                .write_event(Event::Text(BytesText::from_plain_str(&self.input_file)))
                .unwrap();
            writer.write_event(Event::End(BytesEnd::borrowed(b"property"))).unwrap();

            writer.write_event(Event::End(BytesEnd::borrowed(b"chain"))).unwrap();
        }

        let mut playlist0 = BytesStart::borrowed_name(b"playlist");
        playlist0.push_attribute(("id", "playlist0"));
        writer.write_event(Event::Start(playlist0)).unwrap();

        for (i, (start, end)) in self.timestamps.iter().enumerate() {
            let mut entry = BytesStart::borrowed_name(b"entry");
            entry.push_attribute(("producer", &format!("chain{}", i)[..]));
            entry.push_attribute(("in", &format_time(*start)[..]));
            entry.push_attribute(("out", &format_time(*end)[..]));
            writer.write_event(Event::Empty(entry)).unwrap();
        }
        writer.write_event(Event::End(BytesEnd::borrowed(b"playlist"))).unwrap();

        let mut tractor = BytesStart::borrowed_name(b"tractor");
        tractor.push_attribute(("id", "tractor0"));
        tractor.push_attribute(("in", "00:00:00.000"));
        tractor.push_attribute(("out", &total_duration[..]));
        writer.write_event(Event::Start(tractor)).unwrap();

        writer.write_event(Event::End(BytesEnd::borrowed(b"mlt"))).unwrap();
    }
}

fn format_time(seconds: f64) -> String {
    let hours = (seconds / 3600.0).floor() as u32;
    let minutes = ((seconds % 3600.0) / 60.0).floor() as u32;
    let seconds = seconds % 60.0;
    format!("{:02}:{:02}:{:06.3}", hours, minutes, seconds)
}
