use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write, BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use xml::reader::{EventReader, XmlEvent};
use vtc::{Timecode, rates};


fn get_video_metadata(input_video: &str) -> HashMap<String, Value> {
    let ffprobe_cmd = format!(
        "ffprobe -v quiet -print_format json -show_streams {}",
        input_video
    );
    let output = Command::new("bash")
        .arg("-c")
        .arg(ffprobe_cmd)
        .stdout(Stdio::piped())
        .output()
        .expect("failed to execute process");
    let metadata: HashMap<String, Value> =
        serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    return metadata;
}


fn extract_frame_rate(metadata: &HashMap<String, Value>) -> i32 {
    let mut frame_rate_str = metadata["streams"][0]["r_frame_rate"].as_str().unwrap();
    let fps: i32;

    if frame_rate_str == "0/0" {
        frame_rate_str = metadata["streams"][1]["r_frame_rate"].as_str().unwrap();
    }

    if frame_rate_str.contains("/") {
        let parts: Vec<&str> = frame_rate_str.split('/').collect();
        let numerator = parts[0].parse::<f32>().unwrap();
        let denominator = parts[1].parse::<f32>().unwrap();

        fps = (numerator / denominator).floor() as i32;

    } else {
        fps = frame_rate_str.parse::<i32>().unwrap();
    }
    return fps
}



fn xml_parser(xml_path: String, timecodes: &mut Vec<String>) {
    let file = fs::File::open(xml_path).unwrap();
    let file = BufReader::new(file);
    let parser = EventReader::new(file);
    let mut text = String::new();

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, .. }) if name.local_name == "artwork_time" => {
                text.clear();
            }
            Ok(XmlEvent::EndElement { name })
                if name.local_name == "artwork_time" && text.chars().count() == 11 => {
                    timecodes.push(text.trim().to_string());
                }
            Ok(XmlEvent::Characters(c)) => {
                text.push_str(&c);
            }
            _ => {}
        }
    }

}

// Format timecodes with https://github.com/opencinemac/vtc-rs
fn format_timecodes(timecodes: &[String], fps: &i32) -> Vec<String> {
    let rate = match *fps {
        23 => rates::F23_98,
        24 => rates::F24,
        25 => rates::F24,
        29 => rates::F29_97_NDF,
        // 29 => rates::F29_97_DF // NTSC drop frame,
        30 => rates::F30,
        _ => panic!("Unsupported frame rate"),
    };

    // println!("{}", rate);

    timecodes
        .iter()
        .map(|timecode| {
            let tc = Timecode::with_frames(timecode, rate).unwrap();
            
            format!("{}", tc.runtime(3))


        })
        .collect()
}



fn generate_preview_image(input_video: &str, timecode: String, output_folder: Arc<Mutex<String>>) {

    let output_image = output_folder.lock().unwrap().clone() + &format!("/{}.jpg", timecode);

    let cmd = Command::new("ffmpeg")
        .arg("-ss").arg(timecode.clone())
        .arg("-i").arg(input_video)
        .arg("-frames:v").arg("1")
        .arg("-y")
        .arg(&output_image)
        // .stdout(Stdio::piped())
        // .stderr(Stdio::piped())
        .output()
        .expect("failed to execute process");

    // io::stdout().write_all(&cmd.stdout).unwrap();

    
    if cmd.status.success() {
        println!("Printed the preview image for {}", timecode);
    } else {
        println!("Oops, something went wrong");
        io::stderr().write_all(&cmd.stderr).unwrap();
    }
    // let output = cmd.wait_with_output().unwrap();
    // if !output.status.success() {
    //     let stderr = String::from_utf8(output.stderr).unwrap();
    //     println!("Error: {}", stderr);
    // } else {
    //     println!("Printed the preview image for {}", timecode);
    // }

}



fn main() {
    
    // Main video from command line input 
    let mut input_video = String::new();
    println!("Enter the path to the video file:");
    let stdin = std::io::stdin();
    stdin.lock().read_line(&mut input_video).unwrap();
    input_video = input_video.trim().to_string();

    // Get metadata of the main video (for the framerate) with ffprobe
    let metadata = get_video_metadata(&input_video);

    // Translate the framerate to the logic number we will need
    let fps = extract_frame_rate(&metadata);


    // Get xml -> Continue only if the app retrieves succesfully the metadata of the main video
    let mut xml_file_path = String::new();
    println!("Enter the path to the XML file: ");
    let stdin = std::io::stdin();
    stdin.lock().read_line(&mut xml_file_path).unwrap();
    xml_file_path = xml_file_path.trim().to_string();

    // Get timecodes
    let mut timecodes = vec![];
    xml_parser(xml_file_path, &mut timecodes);
    let formatted_timecodes: Vec<String> = format_timecodes(&timecodes, &fps);
    println!("{:?}", timecodes);


    // Set output folder
    let output_folder = "./tmp-previews".to_string(); 
    fs::create_dir_all(&output_folder).unwrap();


    // Generate the previews
    let shared_output_folder = Arc::new(Mutex::new(output_folder));
    let handles: Vec<_> = formatted_timecodes
        .into_iter()
        .map(|timecode| {
            let input_video = input_video.clone();
            let shared_output_folder = shared_output_folder.clone();
            std::thread::spawn(move || {
                generate_preview_image(
                    &input_video,
                    timecode,
                    shared_output_folder,
                );
            })
        })
        .collect();

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }
    
}
